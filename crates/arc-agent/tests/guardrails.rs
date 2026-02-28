use arc_agent::cli::default_model;
use arc_agent::{AnthropicProfile, GeminiProfile, OpenAiProfile, ProviderProfile};
use arc_llm::catalog;
use arc_llm::provider::Provider;

#[test]
fn every_default_model_exists_in_catalog() {
    for &provider in Provider::ALL {
        let model = default_model(provider);
        assert!(
            catalog::get_model_info(model).is_some(),
            "default_model for {:?} is '{}' but it is not in the catalog",
            provider,
            model
        );
    }
}

#[test]
fn profile_context_window_matches_catalog_for_default_models() {
    for &provider in Provider::ALL {
        let model = default_model(provider);
        let catalog_info = catalog::get_model_info(model).unwrap_or_else(|| {
            panic!(
                "default_model '{}' for {:?} not in catalog",
                model, provider
            )
        });

        let profile: Box<dyn ProviderProfile> = match provider {
            Provider::OpenAi => Box::new(OpenAiProfile::new(model)),
            Provider::Kimi | Provider::Zai | Provider::Minimax | Provider::Inception => {
                Box::new(OpenAiProfile::new(model).with_provider(provider))
            }
            Provider::Gemini => Box::new(GeminiProfile::new(model)),
            Provider::Anthropic => Box::new(AnthropicProfile::new(model)),
        };

        assert_eq!(
            profile.context_window_size(),
            catalog_info.context_window as usize,
            "context_window_size mismatch for {:?} model '{}': profile={} catalog={}",
            provider,
            model,
            profile.context_window_size(),
            catalog_info.context_window as usize
        );
    }
}
