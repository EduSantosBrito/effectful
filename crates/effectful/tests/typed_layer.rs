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

  mod when_requirement_present {
    use super::*;

    #[test]
    fn builds_layer() {
      let layer = TypedLayer::from_fn(|| Ok(42)).requiring("Config");

      let ctx = ServiceContext::empty().add(Config { _port: 8080 });
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

mod build_without_dependency_validation {
  use super::*;

  #[test]
  fn remains_compatible() {
    let layer: TypedLayer<i32, LayerError> = TypedLayer::from_fn(|| Ok(42));

    let result = layer.build();

    assert_eq!(result, Ok(42));
  }
}
