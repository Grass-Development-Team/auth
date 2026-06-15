use cache::{Cache, MokaCache};
use token::services::{RegisterTokenService, SessionLookup, SessionService};

fn moka() -> Cache {
    Cache::Moka(MokaCache::new(10_000))
}

#[tokio::test]
async fn session_create_resolve_delete() {
    let c = moka();
    let sid = SessionService::create(&c, 42, 600).await.unwrap();
    assert!(matches!(
        SessionService::resolve(&c, &sid).await.unwrap(),
        SessionLookup::Valid(_)
    ));
    SessionService::delete(&c, &sid).await.unwrap();
    assert!(matches!(
        SessionService::resolve(&c, &sid).await.unwrap(),
        SessionLookup::Missing
    ));
}

#[tokio::test]
async fn session_delete_all_by_uid() {
    let c = moka();
    let s1 = SessionService::create(&c, 7, 600).await.unwrap();
    let s2 = SessionService::create(&c, 7, 600).await.unwrap();
    SessionService::delete_all_by_uid(&c, 7).await.unwrap();
    assert!(matches!(
        SessionService::resolve(&c, &s1).await.unwrap(),
        SessionLookup::Missing
    ));
    assert!(matches!(
        SessionService::resolve(&c, &s2).await.unwrap(),
        SessionLookup::Missing
    ));
}

#[tokio::test]
async fn register_token_issue_reuse_consume() {
    let c = moka();
    let lease1 = RegisterTokenService::issue_or_reuse_for_user(&c, 1, "a@b.c", 3600, 60)
        .await
        .unwrap();
    let lease2 = RegisterTokenService::issue_or_reuse_for_user(&c, 1, "a@b.c", 3600, 60)
        .await
        .unwrap();
    assert_eq!(lease1.token, lease2.token, "同用户同邮箱应复用");
    let consumed = RegisterTokenService::consume(&c, &lease1.token)
        .await
        .unwrap();
    assert_eq!(consumed.unwrap().uid, 1);
    assert!(
        RegisterTokenService::consume(&c, &lease1.token)
            .await
            .unwrap()
            .is_none()
    );
}
