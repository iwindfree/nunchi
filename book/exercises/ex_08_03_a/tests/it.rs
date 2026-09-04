use ex_08_03_a::{run_two, Config};

#[tokio::test]
async fn two_tasks_share_config() {
    let c = Config { name: "web".to_string(), budget: 4000 };
    let (a, b) = run_two(c).await;
    assert_eq!(a, "web:4000");
    assert_eq!(b, "web!");
}
