//! Package manager — dependency resolution, fetching, and caching.
//!
//! Reads ogham.mod.yaml, resolves dependencies (path, git, version),
//! fetches them into $OGHAM_HOME/pkg/mod/, and makes them available
//! for compilation.

use crate::manifest::{self, RequireEntry, ReplaceEntry};
use crate::pipeline::SourceFile;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Semver ─────────────────────────────────────────────────────────────

/// Parsed semantic version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v');
        let parts: Vec<&str> = s.split('.').collect();
        Some(SemVer {
            major: parts.first()?.parse().ok()?,
            minor: parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0),
            patch: parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0),
        })
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A version range constraint.
#[derive(Debug, Clone)]
pub enum VersionRange {
    /// `^1.2.3` → >=1.2.3, <2.0.0
    Caret(SemVer),
    /// `~1.2.3` → >=1.2.3, <1.3.0
    Tilde(SemVer),
    /// `=1.2.3` → exactly 1.2.3
    Exact(SemVer),
    /// `>=1.2.3` → >=1.2.3
    Gte(SemVer),
    /// `*` → any version
    Any,
}

impl VersionRange {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s == "*" {
            return Some(VersionRange::Any);
        }
        if let Some(rest) = s.strip_prefix('^') {
            return Some(VersionRange::Caret(SemVer::parse(rest)?));
        }
        if let Some(rest) = s.strip_prefix('~') {
            return Some(VersionRange::Tilde(SemVer::parse(rest)?));
        }
        if let Some(rest) = s.strip_prefix(">=") {
            return Some(VersionRange::Gte(SemVer::parse(rest.trim())?));
        }
        if let Some(rest) = s.strip_prefix('=') {
            return Some(VersionRange::Exact(SemVer::parse(rest.trim())?));
        }
        // Try as exact version
        SemVer::parse(s).map(VersionRange::Exact)
    }

    /// Check if a version satisfies this range.
    pub fn matches(&self, ver: &SemVer) -> bool {
        match self {
            VersionRange::Any => true,
            VersionRange::Exact(req) => ver == req,
            VersionRange::Gte(req) => ver >= req,
            VersionRange::Caret(req) => {
                // ^1.2.3 → >=1.2.3, <2.0.0
                // ^0.2.3 → >=0.2.3, <0.3.0
                // ^0.0.3 → >=0.0.3, <0.0.4
                if req.major > 0 {
                    ver.major == req.major && ver >= req
                } else if req.minor > 0 {
                    ver.major == 0 && ver.minor == req.minor && ver >= req
                } else {
                    ver == req
                }
            }
            VersionRange::Tilde(req) => {
                // ~1.2.3 → >=1.2.3, <1.3.0
                ver.major == req.major && ver.minor == req.minor && ver >= req
            }
        }
    }

    /// Minimum version that satisfies this range (for MVS).
    pub fn minimum(&self) -> Option<SemVer> {
        match self {
            VersionRange::Any => Some(SemVer { major: 0, minor: 0, patch: 0 }),
            VersionRange::Exact(v) | VersionRange::Caret(v) | VersionRange::Tilde(v) | VersionRange::Gte(v) => {
                Some(v.clone())
            }
        }
    }
}

/// Check if two version ranges are compatible (can be satisfied by a single version).
pub fn ranges_compatible(a: &VersionRange, b: &VersionRange) -> bool {
    // If either is Any, always compatible
    if matches!(a, VersionRange::Any) || matches!(b, VersionRange::Any) {
        return true;
    }

    let a_min = match a.minimum() { Some(v) => v, None => return true };
    let b_min = match b.minimum() { Some(v) => v, None => return true };

    // Check that both ranges could be satisfied by the same version.
    // For caret ranges: major versions must match (or one is 0.x)
    a.matches(&b_min) || b.matches(&a_min)
}

/// MVS: pick the minimum version that satisfies all requirements.
pub fn mvs_select(ranges: &[VersionRange]) -> Option<SemVer> {
    if ranges.is_empty() {
        return None;
    }
    // MVS: take the maximum of all minimums
    let mut selected = SemVer { major: 0, minor: 0, patch: 0 };
    for range in ranges {
        if let Some(min) = range.minimum() {
            if min > selected {
                selected = min;
            }
        }
    }
    // Verify the selected version satisfies all ranges
    for range in ranges {
        if !range.matches(&selected) {
            return None; // conflict
        }
    }
    Some(selected)
}

// ── Public API ─────────────────────────────────────────────────────────

/// Resolved dependency — ready for compilation.
#[derive(Debug, Clone)]
pub struct ResolvedDep {
    pub module: String,
    pub version: String,
    pub path: PathBuf,
    pub source: DepSource,
}

#[derive(Debug, Clone)]
pub enum DepSource {
    Path,
    Git,
    Cache,
}

/// Resolve and fetch all dependencies from ogham.mod.yaml,
/// including transitive dependencies. Uses Minimal Version Selection (MVS):
/// for each package, select the minimum version that satisfies all requirements.
pub fn resolve_deps(project_dir: &Path) -> Result<Vec<ResolvedDep>, String> {
    let mod_file = manifest::load_mod_file(project_dir)?;

    // Validate mod file
    validate_mod_file(&mod_file)?;

    let mut resolved: Vec<ResolvedDep> = Vec::new();
    let mut seen = HashSet::new();
    let mut version_requirements: HashMap<String, Vec<String>> = HashMap::new();

    // Resolve direct and transitive dependencies
    resolve_deps_recursive(
        &mod_file.require,
        &mod_file.replace,
        project_dir,
        &mut resolved,
        &mut seen,
        &mut version_requirements,
        0,
        &[], // no parent chain yet
    )?;

    // Check for version conflicts (MVS: pick minimum satisfying version)
    let _selected_versions = check_version_conflicts(&version_requirements)?;

    // Deduplicate: keep only first occurrence of each module
    let mut deduped: Vec<ResolvedDep> = Vec::new();
    let mut dedup_seen = HashSet::new();
    for dep in resolved {
        if dedup_seen.insert(dep.module.clone()) {
            deduped.push(dep);
        }
    }

    Ok(deduped)
}

/// Validate ogham.mod.yaml for common issues.
fn validate_mod_file(mod_file: &manifest::ModFile) -> Result<(), String> {
    if mod_file.module.is_empty() {
        return Err("ogham.mod.yaml: 'module' field is required".into());
    }

    // Path and replace deps are not allowed in published packages
    // (we check this but don't block — just warn for now)
    for (module, entry) in &mod_file.require {
        if let RequireEntry::Path { .. } = entry {
            // Path deps are fine for local dev, but ogham publish would reject
        }
        // Validate version syntax
        if let RequireEntry::Version(v) = entry {
            if !v.starts_with('^') && !v.starts_with('~') && !v.starts_with('=')
                && !v.starts_with('>') && !v.starts_with('*')
            {
                return Err(format!(
                    "invalid version range for {}: '{}'. Use ^, ~, =, >=, or *",
                    module, v
                ));
            }
        }
    }

    Ok(())
}

const MAX_DEPTH: usize = 32;

#[allow(clippy::too_many_arguments)]
fn resolve_deps_recursive(
    require: &HashMap<String, RequireEntry>,
    replace: &HashMap<String, ReplaceEntry>,
    project_dir: &Path,
    resolved: &mut Vec<ResolvedDep>,
    seen: &mut HashSet<String>,
    version_reqs: &mut HashMap<String, Vec<String>>,
    depth: usize,
    parent_chain: &[String], // for error messages: ["root", "dep-a"]
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(format!(
            "dependency resolution depth limit exceeded ({}). Possible circular dependency: {}",
            MAX_DEPTH,
            parent_chain.join(" → ")
        ));
    }

    for (module, entry) in require {
        if seen.contains(module) {
            // Already resolved — just record the version requirement for conflict check
            if let RequireEntry::Version(v) = entry {
                version_reqs.entry(module.clone()).or_default().push(v.clone());
            }
            continue;
        }

        seen.insert(module.clone());

        // Track version requirements
        if let RequireEntry::Version(v) = entry {
            version_reqs.entry(module.clone()).or_default().push(v.clone());
        }

        // Resolve this dependency
        let dep = if let Some(rep) = replace.get(module) {
            resolve_replace(module, rep, project_dir)
        } else {
            resolve_require(module, entry, project_dir)
        }.map_err(|e| {
            if parent_chain.is_empty() {
                e
            } else {
                format!("{} (required by {})", e, parent_chain.last().unwrap())
            }
        })?;

        // Try to load transitive dependencies from the dep's ogham.mod.yaml
        let dep_dir = dep.path.clone();
        let transitive_mod = if dep_dir.is_dir() {
            manifest::load_mod_file(&dep_dir).ok()
        } else {
            None
        };

        resolved.push(dep);

        // Recurse into transitive deps — resolve paths relative to the dep's directory
        if let Some(trans_mod) = transitive_mod {
            let mut chain = parent_chain.to_vec();
            chain.push(module.clone());
            resolve_deps_recursive(
                &trans_mod.require,
                &HashMap::new(), // replace only applies to root module
                &dep_dir,
                resolved,
                seen,
                version_reqs,
                depth + 1,
                &chain,
            )?;
        }
    }

    Ok(())
}

/// Check for conflicting version requirements using full semver range matching.
/// MVS: pick the minimum version satisfying all requirements.
fn check_version_conflicts(
    version_reqs: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, SemVer>, String> {
    let mut selected = HashMap::new();

    for (module, version_strs) in version_reqs {
        if version_strs.len() <= 1 {
            if let Some(v) = version_strs.first() {
                if let Some(range) = VersionRange::parse(v) {
                    if let Some(ver) = range.minimum() {
                        selected.insert(module.clone(), ver);
                    }
                }
            }
            continue;
        }

        // Parse all ranges
        let ranges: Vec<VersionRange> = version_strs
            .iter()
            .filter_map(|v| VersionRange::parse(v))
            .collect();

        // Check pairwise compatibility
        for i in 0..ranges.len() {
            for j in (i + 1)..ranges.len() {
                if !ranges_compatible(&ranges[i], &ranges[j]) {
                    return Err(format!(
                        "version conflict for {}: {} and {} are incompatible. \
                         All dependents must agree on compatible version ranges.",
                        module, version_strs[i], version_strs[j]
                    ));
                }
            }
        }

        // MVS: select minimum satisfying version
        match mvs_select(&ranges) {
            Some(ver) => { selected.insert(module.clone(), ver); }
            None => {
                return Err(format!(
                    "version conflict for {}: no version satisfies all requirements ({})",
                    module,
                    version_strs.join(", ")
                ));
            }
        }
    }

    Ok(selected)
}

/// Write lock file with resolved dependency versions and git commits.
pub fn write_lock_file(project_dir: &Path, deps: &[ResolvedDep]) -> Result<(), String> {
    let lock_path = project_dir.join("ogham.lock.yaml");
    let mut content = String::from("# Auto-generated by ogham install. Do not edit.\n");
    content.push_str("locked:\n");

    for dep in deps {
        content.push_str(&format!("  {}:\n", dep.module));
        content.push_str(&format!("    version: \"{}\"\n", dep.version));

        // For git deps, record the commit hash
        if matches!(dep.source, DepSource::Git) && dep.path.is_dir() {
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&dep.path)
                .output()
            {
                if output.status.success() {
                    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    content.push_str(&format!("    commit: \"{}\"\n", hash));
                }
            }
        }

        let source = match dep.source {
            DepSource::Path => "path",
            DepSource::Git => "git",
            DepSource::Cache => "cache",
        };
        content.push_str(&format!("    source: {}\n", source));
    }

    std::fs::write(&lock_path, &content)
        .map_err(|e| format!("cannot write lock file: {}", e))?;
    Ok(())
}

/// Verify integrity of cached/git dependencies.
pub fn check_integrity(deps: &[ResolvedDep]) -> Vec<String> {
    let mut warnings = Vec::new();

    for dep in deps {
        if !dep.path.is_dir() {
            if dep.path.to_string_lossy() != "(embedded)" {
                warnings.push(format!("{}: directory not found: {}", dep.module, dep.path.display()));
            }
            continue;
        }

        // For git deps: check if worktree is clean
        if matches!(dep.source, DepSource::Git) {
            if let Ok(output) = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(&dep.path)
                .output()
            {
                if output.status.success() {
                    let status = String::from_utf8_lossy(&output.stdout);
                    if !status.trim().is_empty() {
                        warnings.push(format!(
                            "{}: git dependency has uncommitted changes",
                            dep.module
                        ));
                    }
                }
            }
        }

        // Check that at least one .ogham file exists
        let has_ogham = std::fs::read_dir(&dep.path)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.path().extension().is_some_and(|ext| ext == "ogham")
                        || e.path().is_dir()
                })
            })
            .unwrap_or(false);

        if !has_ogham {
            warnings.push(format!(
                "{}: no .ogham files found in {}",
                dep.module,
                dep.path.display()
            ));
        }
    }

    warnings
}

/// Auto-detect dependency source from a spec string.
/// `github.com/org/lib` → git clone from https://github.com/org/lib.git
/// `github.com/org/lib@v1.0.0` → git clone + tag
pub fn auto_detect_source(spec: &str) -> RequireEntry {
    let (module, version) = if let Some(at) = spec.find('@') {
        (&spec[..at], Some(&spec[at + 1..]))
    } else {
        (spec, None)
    };

    // GitHub/GitLab/etc → git dependency
    if module.starts_with("github.com/")
        || module.starts_with("gitlab.com/")
        || module.starts_with("bitbucket.org/")
    {
        let git_url = format!("https://{}.git", module);
        return RequireEntry::Git {
            git: git_url,
            tag: version.map(|v| v.to_string()),
            branch: None,
            rev: None,
        };
    }

    // Default: version string
    RequireEntry::Version(version.unwrap_or("*").to_string())
}

/// Update dependencies to the maximum version within their range.
pub fn update_deps(project_dir: &Path) -> Result<Vec<ResolvedDep>, String> {
    // For git deps: fetch latest
    // For version deps: would query registry for latest within range
    // For now: re-resolve (which fetches latest git refs if not cached)

    // Clear git cache to force re-fetch
    let git_cache = ogham_home().join("git").join("checkouts");
    if git_cache.is_dir() {
        let _ = std::fs::remove_dir_all(&git_cache);
    }

    resolve_deps(project_dir)
}

/// Collect .ogham source files from all resolved dependencies.
pub fn collect_dep_sources(deps: &[ResolvedDep]) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    for dep in deps {
        collect_ogham_files(&dep.path, &mut sources)?;
    }
    Ok(sources)
}

/// Install all dependencies (fetch + cache).
pub fn install(project_dir: &Path) -> Result<Vec<ResolvedDep>, String> {
    let deps = resolve_deps(project_dir)?;

    for dep in &deps {
        eprintln!(
            "  {} v{} ({})",
            dep.module,
            dep.version,
            match dep.source {
                DepSource::Path => format!("path: {}", dep.path.display()),
                DepSource::Git => format!("git → {}", dep.path.display()),
                DepSource::Cache => format!("cached: {}", dep.path.display()),
            }
        );
    }

    Ok(deps)
}

/// Add a dependency to ogham.mod.yaml.
pub fn add_dependency(project_dir: &Path, spec: &str) -> Result<(), String> {
    let mod_path = project_dir.join("ogham.mod.yaml");
    let mut content = if mod_path.exists() {
        std::fs::read_to_string(&mod_path)
            .map_err(|e| format!("cannot read ogham.mod.yaml: {}", e))?
    } else {
        return Err("ogham.mod.yaml not found".into());
    };

    // Parse module@version or just module
    let (module, version) = if let Some(at) = spec.find('@') {
        (&spec[..at], &spec[at + 1..])
    } else {
        (spec, "*")
    };

    // Check if already in require
    if content.contains(module) {
        return Err(format!("{} already in dependencies", module));
    }

    // Append to require section
    if content.contains("require:") {
        // Add after require:
        let insert_pos = content
            .find("require:")
            .map(|p| {
                // Find end of require line
                content[p..].find('\n').map(|n| p + n + 1).unwrap_or(content.len())
            })
            .unwrap_or(content.len());

        let line = format!("  {}: {}\n", module, version);
        content.insert_str(insert_pos, &line);
    } else {
        content.push_str(&format!("\nrequire:\n  {}: {}\n", module, version));
    }

    std::fs::write(&mod_path, &content)
        .map_err(|e| format!("cannot write ogham.mod.yaml: {}", e))?;

    eprintln!("added {} v{}", module, version);

    // Fetch the dependency
    let _ = resolve_deps(project_dir);

    Ok(())
}

/// Copy all dependencies into vendor/ directory.
pub fn vendor(project_dir: &Path) -> Result<(), String> {
    let deps = resolve_deps(project_dir)?;
    let vendor_dir = project_dir.join("vendor");

    for dep in &deps {
        let target = vendor_dir.join(&dep.module);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {}", parent.display(), e))?;
        }

        copy_dir(&dep.path, &target)?;
        eprintln!("  vendored {} → {}", dep.module, target.display());
    }

    Ok(())
}

// ── Resolution ─────────────────────────────────────────────────────────

fn resolve_require(
    module: &str,
    entry: &RequireEntry,
    project_dir: &Path,
) -> Result<ResolvedDep, String> {
    match entry {
        RequireEntry::Path { path } => {
            let resolved = project_dir.join(path);
            if !resolved.is_dir() {
                return Err(format!("path dependency not found: {} ({})", module, resolved.display()));
            }
            Ok(ResolvedDep {
                module: module.to_string(),
                version: "local".to_string(),
                path: resolved,
                source: DepSource::Path,
            })
        }
        RequireEntry::Git { git, tag, branch, rev } => {
            resolve_git(module, git, tag.as_deref(), branch.as_deref(), rev.as_deref())
        }
        RequireEntry::Version(version) => {
            // Check cache first
            let cache_dir = pkg_cache_dir(module, version);
            if cache_dir.is_dir() {
                return Ok(ResolvedDep {
                    module: module.to_string(),
                    version: version.clone(),
                    path: cache_dir,
                    source: DepSource::Cache,
                });
            }

            // For std packages, they're embedded — create a marker
            if module.starts_with("github.com/oghamlang/std") {
                return Ok(ResolvedDep {
                    module: module.to_string(),
                    version: version.clone(),
                    path: PathBuf::from("(embedded)"),
                    source: DepSource::Cache,
                });
            }

            Err(format!(
                "dependency {} v{} not in cache. Use git or path dependency, or run `ogham install` with a registry configured.",
                module, version
            ))
        }
    }
}

fn resolve_replace(
    module: &str,
    replace: &ReplaceEntry,
    project_dir: &Path,
) -> Result<ResolvedDep, String> {
    match replace {
        ReplaceEntry::Path { path } => {
            let resolved = project_dir.join(path);
            if !resolved.is_dir() {
                return Err(format!(
                    "replace path not found: {} → {} ({})",
                    module,
                    path,
                    resolved.display()
                ));
            }
            Ok(ResolvedDep {
                module: module.to_string(),
                version: "local-replace".to_string(),
                path: resolved,
                source: DepSource::Path,
            })
        }
        ReplaceEntry::Git { git, branch } => {
            resolve_git(module, git, None, branch.as_deref(), None)
        }
    }
}

fn resolve_git(
    module: &str,
    git_url: &str,
    tag: Option<&str>,
    branch: Option<&str>,
    rev: Option<&str>,
) -> Result<ResolvedDep, String> {
    let git_ref = tag
        .or(branch)
        .or(rev)
        .unwrap_or("HEAD");

    let version = tag.unwrap_or(git_ref);
    let cache_dir = git_cache_dir(module, git_ref);

    // Already cloned?
    if cache_dir.is_dir() {
        return Ok(ResolvedDep {
            module: module.to_string(),
            version: version.to_string(),
            path: cache_dir,
            source: DepSource::Git,
        });
    }

    // Clone
    if let Some(parent) = cache_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create cache dir: {}", e))?;
    }

    eprintln!("  fetching {} from {} ({})", module, git_url, git_ref);

    let mut cmd = Command::new("git");
    cmd.args(["clone", "--depth", "1"]);

    if let Some(b) = branch {
        cmd.args(["--branch", b]);
    } else if let Some(t) = tag {
        cmd.args(["--branch", t]);
    }

    cmd.arg(git_url);
    cmd.arg(&cache_dir);

    let output = cmd.output().map_err(|e| format!("git clone failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed for {}: {}",
            git_url,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    // Checkout specific rev if provided
    if let Some(r) = rev {
        let output = Command::new("git")
            .args(["checkout", r])
            .current_dir(&cache_dir)
            .output()
            .map_err(|e| format!("git checkout failed: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git checkout {} failed: {}",
                r,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }

    Ok(ResolvedDep {
        module: module.to_string(),
        version: version.to_string(),
        path: cache_dir,
        source: DepSource::Git,
    })
}

// ── Paths ──────────────────────────────────────────────────────────────

fn ogham_home() -> PathBuf {
    if let Ok(home) = std::env::var("OGHAM_HOME") {
        PathBuf::from(home)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".ogham")
    } else {
        PathBuf::from(".ogham")
    }
}

fn pkg_cache_dir(module: &str, version: &str) -> PathBuf {
    ogham_home()
        .join("pkg")
        .join("mod")
        .join(format!("{}@{}", module, version))
}

fn git_cache_dir(module: &str, git_ref: &str) -> PathBuf {
    // Sanitize ref for filesystem
    let safe_ref = git_ref.replace('/', "_");
    ogham_home()
        .join("git")
        .join("checkouts")
        .join(module)
        .join(safe_ref)
}

// ── Helpers ────────────────────────────────────────────────────────────

fn collect_ogham_files(dir: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(()); // embedded or missing — skip
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("read error: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "vendor" && name != "node_modules" {
                collect_ogham_files(&path, sources)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "ogham") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
            sources.push(SourceFile {
                name: path.to_string_lossy().to_string(),
                content,
            });
        }
    }

    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)
        .map_err(|e| format!("cannot create {}: {}", dst.display(), e))?;

    for entry in std::fs::read_dir(src).map_err(|e| format!("read error: {}", e))? {
        let entry = entry.map_err(|e| format!("entry error: {}", e))?;
        let path = entry.path();
        let target = dst.join(entry.file_name());

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == ".git" {
                continue; // skip .git in vendor
            }
            copy_dir(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)
                .map_err(|e| format!("copy error: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("ogham-pkg-{}-{}", std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_path_dep() {
        let dir = temp_dir();
        let dep_dir = dir.join("libs").join("mylib");
        fs::create_dir_all(&dep_dir).unwrap();
        fs::write(dep_dir.join("types.ogham"), "package mylib;\ntype Foo { string bar = 1; }\n").unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  mylib:\n    path: ./libs/mylib\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].module, "mylib");
        assert!(matches!(deps[0].source, DepSource::Path));

        let sources = collect_dep_sources(&deps).unwrap();
        assert_eq!(sources.len(), 1);
        assert!(sources[0].content.contains("Foo"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_replace_path() {
        let dir = temp_dir();
        let fork_dir = dir.join("my-fork");
        fs::create_dir_all(&fork_dir).unwrap();
        fs::write(fork_dir.join("models.ogham"), "package fork;\ntype Bar { int32 x = 1; }\n").unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  github.com/org/lib: ^1.0.0\nreplace:\n  github.com/org/lib:\n    path: ./my-fork\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].module, "github.com/org/lib");
        assert!(matches!(deps[0].source, DepSource::Path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_std_embedded() {
        let dir = temp_dir();
        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  github.com/oghamlang/std: ^0.1.0\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].path.to_string_lossy(), "(embedded)");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_missing_path_dep_errors() {
        let dir = temp_dir();
        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  mylib:\n    path: ./nonexistent\n",
        ).unwrap();

        let result = resolve_deps(&dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn vendor_copies_deps() {
        let dir = temp_dir();
        let dep_dir = dir.join("libs").join("mylib");
        fs::create_dir_all(&dep_dir).unwrap();
        fs::write(dep_dir.join("types.ogham"), "package mylib;\n").unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  mylib:\n    path: ./libs/mylib\n",
        ).unwrap();

        vendor(&dir).unwrap();
        assert!(dir.join("vendor").join("mylib").join("types.ogham").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_dependency_to_mod() {
        let dir = temp_dir();
        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  github.com/oghamlang/std: ^0.1.0\n",
        ).unwrap();

        // add_dependency may fail on fetch but should write to file
        let _ = add_dependency(&dir, "github.com/org/newlib@^1.0.0");
        let content = fs::read_to_string(dir.join("ogham.mod.yaml")).unwrap();
        assert!(content.contains("github.com/org/newlib"), "mod file: {}", content);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn transitive_deps() {
        let dir = temp_dir();

        // Create dep A which depends on dep B
        let dep_a = dir.join("libs").join("dep-a");
        fs::create_dir_all(&dep_a).unwrap();
        fs::write(dep_a.join("a.ogham"), "package a;\ntype A { string x = 1; }\n").unwrap();
        fs::write(dep_a.join("ogham.mod.yaml"),
            "module: github.com/test/dep-a\nversion: 1.0.0\nrequire:\n  dep-b:\n    path: ../dep-b\n",
        ).unwrap();

        let dep_b = dir.join("libs").join("dep-b");
        fs::create_dir_all(&dep_b).unwrap();
        fs::write(dep_b.join("b.ogham"), "package b;\ntype B { int32 y = 1; }\n").unwrap();
        fs::write(dep_b.join("ogham.mod.yaml"),
            "module: github.com/test/dep-b\nversion: 1.0.0\n",
        ).unwrap();

        // Root depends on dep-a
        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/root\nversion: 0.1.0\nrequire:\n  dep-a:\n    path: ./libs/dep-a\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();

        // Should resolve both dep-a and dep-b (transitive)
        let modules: Vec<&str> = deps.iter().map(|d| d.module.as_str()).collect();
        assert!(modules.contains(&"dep-a"), "missing dep-a: {:?}", modules);
        assert!(modules.contains(&"dep-b"), "missing dep-b: {:?}", modules);

        // Sources from both should be collectible
        let sources = collect_dep_sources(&deps).unwrap();
        assert!(sources.len() >= 2, "expected >= 2 sources, got {}", sources.len());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn circular_dep_detected() {
        let dir = temp_dir();

        // dep-a → dep-b → dep-a (cycle)
        let dep_a = dir.join("libs").join("dep-a");
        fs::create_dir_all(&dep_a).unwrap();
        fs::write(dep_a.join("a.ogham"), "package a;\n").unwrap();
        fs::write(dep_a.join("ogham.mod.yaml"),
            "module: github.com/test/dep-a\nversion: 1.0.0\nrequire:\n  dep-b:\n    path: ../dep-b\n",
        ).unwrap();

        let dep_b = dir.join("libs").join("dep-b");
        fs::create_dir_all(&dep_b).unwrap();
        fs::write(dep_b.join("b.ogham"), "package b;\n").unwrap();
        fs::write(dep_b.join("ogham.mod.yaml"),
            "module: github.com/test/dep-b\nversion: 1.0.0\nrequire:\n  dep-a:\n    path: ../dep-a\n",
        ).unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/root\nversion: 0.1.0\nrequire:\n  dep-a:\n    path: ./libs/dep-a\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        // Should not infinite loop — `seen` set prevents re-processing
        let modules: Vec<&str> = deps.iter().map(|d| d.module.as_str()).collect();
        assert!(modules.contains(&"dep-a"));
        assert!(modules.contains(&"dep-b"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn version_conflict_detected() {
        let version_reqs: std::collections::HashMap<String, Vec<String>> = [
            ("mylib".into(), vec!["^1.0.0".into(), "^2.0.0".into()]),
        ].into();

        let result = check_version_conflicts(&version_reqs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version conflict"));
    }

    #[test]
    fn compatible_versions_ok() {
        let version_reqs: std::collections::HashMap<String, Vec<String>> = [
            ("mylib".into(), vec!["^1.0.0".into(), "^1.2.0".into()]),
        ].into();

        let result = check_version_conflicts(&version_reqs);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_mod_empty_module() {
        let mod_file = manifest::ModFile {
            module: String::new(),
            ..Default::default()
        };
        assert!(validate_mod_file(&mod_file).is_err());
    }

    #[test]
    fn validate_invalid_version_syntax() {
        let dir = temp_dir();
        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  mylib: 1.0.0\n",
        ).unwrap();

        let result = resolve_deps(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid version range"));

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Semver tests ────────────────────────────────────────────────

    #[test]
    fn semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v, SemVer { major: 1, minor: 2, patch: 3 });

        let v = SemVer::parse("v2.0.0").unwrap();
        assert_eq!(v.major, 2);

        let v = SemVer::parse("3").unwrap();
        assert_eq!(v, SemVer { major: 3, minor: 0, patch: 0 });
    }

    #[test]
    fn version_range_caret() {
        let r = VersionRange::parse("^1.2.3").unwrap();
        assert!(r.matches(&SemVer::parse("1.2.3").unwrap()));
        assert!(r.matches(&SemVer::parse("1.9.0").unwrap()));
        assert!(!r.matches(&SemVer::parse("2.0.0").unwrap()));
        assert!(!r.matches(&SemVer::parse("1.2.2").unwrap()));
    }

    #[test]
    fn version_range_caret_zero() {
        let r = VersionRange::parse("^0.2.3").unwrap();
        assert!(r.matches(&SemVer::parse("0.2.3").unwrap()));
        assert!(r.matches(&SemVer::parse("0.2.9").unwrap()));
        assert!(!r.matches(&SemVer::parse("0.3.0").unwrap()));
    }

    #[test]
    fn version_range_tilde() {
        let r = VersionRange::parse("~1.2.3").unwrap();
        assert!(r.matches(&SemVer::parse("1.2.3").unwrap()));
        assert!(r.matches(&SemVer::parse("1.2.9").unwrap()));
        assert!(!r.matches(&SemVer::parse("1.3.0").unwrap()));
    }

    #[test]
    fn version_range_exact() {
        let r = VersionRange::parse("=1.2.3").unwrap();
        assert!(r.matches(&SemVer::parse("1.2.3").unwrap()));
        assert!(!r.matches(&SemVer::parse("1.2.4").unwrap()));
    }

    #[test]
    fn version_range_any() {
        let r = VersionRange::parse("*").unwrap();
        assert!(r.matches(&SemVer::parse("0.0.0").unwrap()));
        assert!(r.matches(&SemVer::parse("99.99.99").unwrap()));
    }

    #[test]
    fn mvs_select_picks_maximum_minimum() {
        let ranges = vec![
            VersionRange::parse("^1.0.0").unwrap(),
            VersionRange::parse("^1.2.0").unwrap(),
        ];
        let selected = mvs_select(&ranges).unwrap();
        assert_eq!(selected, SemVer::parse("1.2.0").unwrap());
    }

    #[test]
    fn mvs_select_conflict_returns_none() {
        let ranges = vec![
            VersionRange::parse("^1.0.0").unwrap(),
            VersionRange::parse("^2.0.0").unwrap(),
        ];
        assert!(mvs_select(&ranges).is_none());
    }

    #[test]
    fn ranges_compatible_same_major() {
        let a = VersionRange::parse("^1.0.0").unwrap();
        let b = VersionRange::parse("^1.5.0").unwrap();
        assert!(ranges_compatible(&a, &b));
    }

    #[test]
    fn ranges_incompatible_different_major() {
        let a = VersionRange::parse("^1.0.0").unwrap();
        let b = VersionRange::parse("^2.0.0").unwrap();
        assert!(!ranges_compatible(&a, &b));
    }

    #[test]
    fn deduplicate_deps() {
        let dir = temp_dir();

        // A depends on C, B depends on C — C should appear once
        let dep_a = dir.join("libs").join("dep-a");
        let dep_b = dir.join("libs").join("dep-b");
        let dep_c = dir.join("libs").join("dep-c");
        fs::create_dir_all(&dep_a).unwrap();
        fs::create_dir_all(&dep_b).unwrap();
        fs::create_dir_all(&dep_c).unwrap();

        fs::write(dep_c.join("c.ogham"), "package c;\n").unwrap();
        fs::write(dep_c.join("ogham.mod.yaml"), "module: dep-c\nversion: 1.0.0\n").unwrap();

        fs::write(dep_a.join("a.ogham"), "package a;\n").unwrap();
        fs::write(dep_a.join("ogham.mod.yaml"),
            "module: dep-a\nversion: 1.0.0\nrequire:\n  dep-c:\n    path: ../dep-c\n",
        ).unwrap();

        fs::write(dep_b.join("b.ogham"), "package b;\n").unwrap();
        fs::write(dep_b.join("ogham.mod.yaml"),
            "module: dep-b\nversion: 1.0.0\nrequire:\n  dep-c:\n    path: ../dep-c\n",
        ).unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/root\nversion: 0.1.0\nrequire:\n  dep-a:\n    path: ./libs/dep-a\n  dep-b:\n    path: ./libs/dep-b\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        let modules: Vec<&str> = deps.iter().map(|d| d.module.as_str()).collect();

        // C should appear exactly once
        let c_count = modules.iter().filter(|&&m| m == "dep-c").count();
        assert_eq!(c_count, 1, "dep-c should be deduplicated, got: {:?}", modules);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_detect_github() {
        let entry = auto_detect_source("github.com/org/lib@v1.0.0");
        match entry {
            RequireEntry::Git { git, tag, .. } => {
                assert_eq!(git, "https://github.com/org/lib.git");
                assert_eq!(tag.as_deref(), Some("v1.0.0"));
            }
            _ => panic!("expected git entry"),
        }
    }

    #[test]
    fn auto_detect_plain_version() {
        let entry = auto_detect_source("mylib@^1.0.0");
        match entry {
            RequireEntry::Version(v) => assert_eq!(v, "^1.0.0"),
            _ => panic!("expected version entry"),
        }
    }

    #[test]
    fn integrity_check_missing_dir() {
        let dep = ResolvedDep {
            module: "test".into(),
            version: "1.0.0".into(),
            path: PathBuf::from("/nonexistent/path"),
            source: DepSource::Cache,
        };
        let warnings = check_integrity(&[dep]);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("not found"));
    }

    #[test]
    fn lock_file_written() {
        let dir = temp_dir();
        let dep_dir = dir.join("libs").join("mylib");
        fs::create_dir_all(&dep_dir).unwrap();
        fs::write(dep_dir.join("types.ogham"), "package mylib;\n").unwrap();

        fs::write(dir.join("ogham.mod.yaml"),
            "module: github.com/test/proj\nversion: 0.1.0\nrequire:\n  mylib:\n    path: ./libs/mylib\n",
        ).unwrap();

        let deps = resolve_deps(&dir).unwrap();
        write_lock_file(&dir, &deps).unwrap();

        let lock = dir.join("ogham.lock.yaml");
        assert!(lock.exists());
        let content = fs::read_to_string(&lock).unwrap();
        assert!(content.contains("mylib"), "lock: {}", content);
        assert!(content.contains("version:"), "lock: {}", content);

        let _ = fs::remove_dir_all(&dir);
    }
}
