// @generated
pub mod oghamproto {
    #[cfg(feature = "oghamproto-common")]
    // @@protoc_insertion_point(attribute:oghamproto.common)
    pub mod common {
        include!("oghamproto/common/oghamproto.common.rs");
        // @@protoc_insertion_point(oghamproto.common)
    }
    #[cfg(feature = "oghamproto-compiler")]
    // @@protoc_insertion_point(attribute:oghamproto.compiler)
    pub mod compiler {
        include!("oghamproto/compiler/oghamproto.compiler.rs");
        // @@protoc_insertion_point(oghamproto.compiler)
    }
    #[cfg(feature = "oghamproto-ir")]
    // @@protoc_insertion_point(attribute:oghamproto.ir)
    pub mod ir {
        include!("oghamproto/ir/oghamproto.ir.rs");
        // @@protoc_insertion_point(oghamproto.ir)
    }
}