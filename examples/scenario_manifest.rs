use flow::ScenarioManifest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: ScenarioManifest = serde_json::from_str(include_str!(
        "../contracts/examples/scenario-manifest.v1.example.json"
    ))?;
    manifest.validate()?;

    println!(
        "{} pins fixture {} at {} with canonical SHA-256 {}",
        manifest.scenario_id,
        manifest.fixture_id,
        manifest.fixture_version,
        manifest.canonical_sha256()?
    );
    Ok(())
}
