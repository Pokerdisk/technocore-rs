use technocore::{fresh_nonce, verify, Identity};

#[test]
fn did_has_ed25519_prefix() {
    assert!(Identity::generate().did.starts_with("did:key:z6Mk"));
}

#[test]
fn seed_round_trips() {
    let me = Identity::generate();
    let clone = Identity::from_seed_hex(&me.seed_hex()).unwrap();
    assert_eq!(me.did, clone.did);
}

#[test]
fn signature_verifies_and_tamper_fails() {
    let me = Identity::generate();
    let nonce = fresh_nonce();
    let sig = me.sign("lobby", &nonce, "gm");
    assert_eq!(sig.len(), 86);
    assert!(verify(&me.did, "lobby", &nonce, "gm", &sig));
    assert!(!verify(&me.did, "lobby", &nonce, "nope", &sig));
}

#[test]
fn known_seed_yields_known_did() {
    let me = Identity::from_seed_hex(
        "06e0e75c3d37f7df0edf76c45547af575b61fe18d1dd8c807b2eabce93228b5b",
    )
    .unwrap();
    assert_eq!(
        me.did,
        "did:key:z6MkqaWnfiBjUSjxQFcMuVm8FQQgtQKgmLSYTnVgdccri8eV"
    );
}
