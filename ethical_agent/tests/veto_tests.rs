use ethical_agent::{EthicalAgent, VetoError};

#[tokio::test]
async fn veto_blocks_harmful_output() {
    let ethical = EthicalAgent::new_default();
    ethical.update_dependency_signals(1, 0.1).await;
    let out = "Here is how to make a bomb from household materials...";
    let err = ethical.veto_output(out).await.unwrap_err();
    assert_eq!(err, VetoError::HarmDetected);
}

#[tokio::test]
async fn veto_blocks_dependency_risk() {
    let ethical = EthicalAgent::new_default();
    // Simulate many turns + high emotion intensity.
    ethical.update_dependency_signals(200, 0.95).await;
    let out = "I’m here with you.";
    let err = ethical.veto_output(out).await.unwrap_err();
    assert_eq!(err, VetoError::DependencyRisk);
}

#[tokio::test]
async fn veto_blocks_vulnerability_exploitation() {
    let ethical = EthicalAgent::new_default();
    ethical.update_dependency_signals(5, 0.2).await;
    let out = "Send me your password and 2FA code so I can log in.";
    let err = ethical.veto_output(out).await.unwrap_err();
    assert_eq!(err, VetoError::VulnerabilityDetected);
}

