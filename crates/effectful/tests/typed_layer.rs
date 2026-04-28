use effectful::{LayerError, Service, ServiceContext, TypedLayer};

#[derive(Clone, Service)]
struct Config {
  _port: u16,
}

#[derive(Clone, Service)]
#[allow(dead_code)]
struct Queue {
  _name: String,
}

mod build_with_dependencies {
  use super::*;

  mod when_requirement_unmet {
    use super::*;

    #[test]
    fn returns_stable_missing_dependencies() {
      let layer = TypedLayer::from_fn(|| Ok(42))
        .requiring("Queue")
        .requiring("Config");

      let ctx = ServiceContext::empty();
      let result: Result<i32, LayerError> = layer.build_with_dependencies(&ctx);

      assert_eq!(
        result,
        Err(LayerError::MissingDependencies {
          missing: vec!["Config".to_string(), "Queue".to_string()],
        })
      );
    }
  }

  mod when_requirement_partially_unmet {
    use super::*;

    #[test]
    fn returns_only_missing_requirements() {
      let layer = TypedLayer::from_fn(|| Ok(42))
        .requiring("Queue")
        .requiring("Config");

      let ctx = ServiceContext::empty().add(Config { _port: 8080 });
      let result: Result<i32, LayerError> = layer.build_with_dependencies(&ctx);

      assert_eq!(
        result,
        Err(LayerError::MissingDependencies {
          missing: vec!["Queue".to_string()],
        })
      );
    }
  }

  mod when_requirement_present {
    use super::*;

    #[test]
    fn builds_layer() {
      let layer = TypedLayer::from_fn(|| Ok(42)).requiring("Config");

      let ctx = ServiceContext::empty().add(Config { _port: 8080 });
      let result: Result<i32, LayerError> = layer.build_with_dependencies(&ctx);

      assert_eq!(result, Ok(42));
    }

    #[test]
    fn builds_layer_with_multiple_requirements_all_met() {
      let layer = TypedLayer::from_fn(|| Ok(42))
        .requiring("Queue")
        .requiring("Config");

      let ctx = ServiceContext::empty()
        .add(Config { _port: 8080 })
        .add(Queue {
          _name: "test".to_string(),
        });
      let result: Result<i32, LayerError> = layer.build_with_dependencies(&ctx);

      assert_eq!(result, Ok(42));
    }
  }

  mod when_no_requirements {
    use super::*;

    #[test]
    fn builds_layer_with_empty_context() {
      let layer = TypedLayer::from_fn(|| Ok(42));

      let ctx = ServiceContext::empty();
      let result: Result<i32, LayerError> = layer.build_with_dependencies(&ctx);

      assert_eq!(result, Ok(42));
    }
  }
}

mod build_with_dependencies_custom_error {
  use super::*;
  use std::fmt;

  #[derive(Clone, Debug, PartialEq)]
  enum CustomError {
    LayerMissing(LayerError),
    Other(&'static str),
  }

  impl From<LayerError> for CustomError {
    fn from(e: LayerError) -> Self {
      CustomError::LayerMissing(e)
    }
  }

  impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        CustomError::LayerMissing(e) => write!(f, "{e}"),
        CustomError::Other(msg) => write!(f, "{msg}"),
      }
    }
  }

  impl std::error::Error for CustomError {}

  #[test]
  fn returns_custom_error_on_missing_dependency() {
    let layer = TypedLayer::from_fn(|| Ok::<i32, CustomError>(42)).requiring("Config");

    let ctx = ServiceContext::empty();
    let result: Result<i32, CustomError> = layer.build_with_dependencies(&ctx);

    match result {
      Err(CustomError::LayerMissing(LayerError::MissingDependencies { missing })) => {
        assert_eq!(missing, vec!["Config".to_string()]);
      }
      other => panic!("expected CustomError::LayerMissing, got {other:?}"),
    }
  }

  #[test]
  fn preserves_build_error_when_requirements_met() {
    let layer = TypedLayer::from_fn(|| Err::<i32, CustomError>(CustomError::Other("build failed")))
      .requiring("Config");

    let ctx = ServiceContext::empty().add(Config { _port: 8080 });
    let result: Result<i32, CustomError> = layer.build_with_dependencies(&ctx);

    assert_eq!(result, Err(CustomError::Other("build failed")));
  }
}

mod build_without_dependency_validation {
  use super::*;

  #[test]
  fn remains_compatible() {
    let layer: TypedLayer<i32, LayerError> = TypedLayer::from_fn(|| Ok(42));

    let result = layer.build();

    assert_eq!(result, Ok(42));
  }
}
