pub use self::gen::*;
pub mod errors;
pub mod repository;
pub mod token;

mod gen {
    pub mod email {
        tonic::include_proto!("email");
    }

    pub mod templating {
        tonic::include_proto!("templating");
    }

    pub mod user {
        tonic::include_proto!("user");
    }
}
