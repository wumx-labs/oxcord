//! `#[command]`: `param(...)` dan `ChoiceParameter` → opsi slash (deskripsi, wajib, pilihan).

use oxcord_framework::{ChoiceParameter as ChoiceTrait, Context};
use oxcord_macros::ChoiceParameter;

#[derive(ChoiceParameter, Debug, PartialEq)]
#[allow(non_camel_case_types)]
enum Notify {
    #[choice("Notify with the reason.")]
    with_reason,
    #[choice("Don't notify.")]
    none,
}

#[derive(ChoiceParameter, Debug, PartialEq)]
enum Plain {
    Alpha,
    Beta,
}

#[oxcord_macros::command(
    slash_command,
    description = "demo",
    param("user", "User to act on", "required"),
    param("reason", "Why")
)]
async fn demo(
    _ctx: Context<'_>,
    user: Option<String>,
    reason: Option<String>,
    notify: Option<Notify>,
    plain: Option<Plain>,
    count: i64,
) -> oxcord_framework::Result<()> {
    let _ = (user, reason, notify, plain, count);
    Ok(())
}

#[test]
fn param_overrides_description_and_required() {
    let cmd = demo();
    let user = cmd.options.iter().find(|o| o.name == "user").unwrap();
    assert_eq!(user.description, "User to act on");
    assert!(user.required, "Option<T> forced required by param(..., \"required\")");
    let reason = cmd.options.iter().find(|o| o.name == "reason").unwrap();
    assert_eq!(reason.description, "Why");
    assert!(!reason.required);
    let count = cmd.options.iter().find(|o| o.name == "count").unwrap();
    assert_eq!(count.description, "count parameter");
    assert!(count.required);
}

#[test]
fn choices_carry_label_and_value() {
    let cmd = demo();
    let notify = cmd.options.iter().find(|o| o.name == "notify").unwrap();
    assert_eq!(notify.choices, vec![
        ("Notify with the reason.".to_owned(), "with_reason".to_owned()),
        ("Don't notify.".to_owned(), "none".to_owned()),
    ]);
    let plain = cmd.options.iter().find(|o| o.name == "plain").unwrap();
    assert_eq!(plain.choices.len(), 2);
    assert_eq!(plain.choices[0], ("Alpha".to_owned(), "Alpha".to_owned()));
    // pilihan dikirim Discord sebagai nilai: harus bisa di-parse balik
    assert_eq!(<Notify as ChoiceTrait>::from_name("with_reason"), Some(Notify::with_reason));
    assert_eq!(<Notify as ChoiceTrait>::from_name("none"), Some(Notify::none));
    // parameter bebas tidak punya pilihan
    let user = cmd.options.iter().find(|o| o.name == "user").unwrap();
    assert!(user.choices.is_empty());
}
