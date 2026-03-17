// Package oghamgen provides the Ogham Plugin SDK for Go.
//
// Build code generation plugins for the Ogham schema language.
//
// Quick start:
//
//	package main
//
//	import (
//		"fmt"
//		"strings"
//
//		"github.com/oghamlang/go/oghamgen"
//		"github.com/oghamlang/go/oghamproto/compiler"
//		"github.com/oghamlang/go/oghamproto/ir"
//	)
//
//	func main() {
//		oghamgen.Run(func(req *compiler.OghamCompileRequest) (*compiler.OghamCompileResponse, error) {
//			resp := &compiler.OghamCompileResponse{}
//			for _, t := range req.Module.Types {
//				resp.Files = append(resp.Files, &compiler.GeneratedFile{
//					Name:    strings.ToLower(t.Name) + ".go",
//					Content: []byte(fmt.Sprintf("package gen\n\ntype %s struct{}\n", t.Name)),
//				})
//			}
//			return resp, nil
//		})
//	}
package oghamgen

import (
	"fmt"
	"io"
	"os"

	"github.com/oghamlang/go/oghamproto/compiler"
	"google.golang.org/protobuf/proto"
)

// Run reads an OghamCompileRequest from stdin, calls the handler,
// and writes the OghamCompileResponse to stdout.
//
// This is the entry point for all Ogham Go plugins.
func Run(handler func(*compiler.OghamCompileRequest) (*compiler.OghamCompileResponse, error)) {
	input, err := io.ReadAll(os.Stdin)
	if err != nil {
		fatal("failed to read stdin: %v", err)
	}

	req := &compiler.OghamCompileRequest{}
	if err := proto.Unmarshal(input, req); err != nil {
		fatal("failed to decode request: %v", err)
	}

	resp, err := handler(req)
	if err != nil {
		// Send error as part of response
		resp = &compiler.OghamCompileResponse{
			Errors: []*compiler.CompileError{{
				Message:  err.Error(),
				Severity: compiler.Severity_ERROR,
			}},
		}
	}

	output, err := proto.Marshal(resp)
	if err != nil {
		fatal("failed to encode response: %v", err)
	}

	if _, err := os.Stdout.Write(output); err != nil {
		fatal("failed to write stdout: %v", err)
	}
}

func fatal(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "oghamgen: "+format+"\n", args...)
	os.Exit(1)
}
