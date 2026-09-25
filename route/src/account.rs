//! The wallet and account selected by Bloom for a scoped Tolly invocation.
use std::cell::Cell;
thread_local! {
    static ACCOUNT: Cell<u32> = const { Cell::new(0) };
    static PREFIX: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

pub fn wallet_param(ctx: &petal::Ctx) -> Result<&str, petal::DispatchResponse> {
    let wallet = petal::route_param(ctx, "bloom.wallet")
        .ok_or_else(|| petal::error(-2, "Bloom did not select a wallet for Tolly"))?;
    crate::wallet::check_wallet_id(wallet)?;
    let number = petal::route_param(ctx, "bloom.account")
        .ok_or_else(|| petal::error(-2, "Bloom did not select an account for Tolly"))?
        .parse::<u32>()
        .map_err(|_| petal::error(-3, "invalid selected account"))?;
    ACCOUNT.with(|account| account.set(number));
    PREFIX.with(|prefix| {
        *prefix.borrow_mut() = petal::route_param(ctx, "bloom.route_prefix").map(str::to_owned)
    });
    Ok(wallet)
}

pub fn number() -> u32 {
    ACCOUNT.with(Cell::get)
}

pub fn link(wallet: &str, path: &str) -> String {
    PREFIX.with(|prefix| match prefix.borrow().as_deref() {
        Some(prefix) => format!("{prefix}{path}"),
        None => format!("wallets/{wallet}/{}/{path}", number()),
    })
}

#[cfg(test)]
pub(crate) fn with_selected_for_test<T>(account: u32, prefix: &str, run: impl FnOnce() -> T) -> T {
    struct Restore(u32, Option<String>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACCOUNT.with(|selected| selected.set(self.0));
            PREFIX.with(|selected| *selected.borrow_mut() = self.1.take());
        }
    }
    let previous_account = ACCOUNT.with(|selected| selected.replace(account));
    let previous_prefix = PREFIX.with(|selected| selected.replace(Some(prefix.to_owned())));
    let _restore = Restore(previous_account, previous_prefix);
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_one_links_and_outbox_confirm_are_numbered() {
        ACCOUNT.with(|account| account.set(1));
        PREFIX.with(|prefix| *prefix.borrow_mut() = Some("wallets/main/1/".into()));
        assert_eq!(link("main", "buy.json"), "wallets/main/1/buy.json");
        assert_eq!(
            crate::tx::confirm_path("main", "ob-1"),
            "wallets/main/1/chains/arc/outbox/pending/ob-1/confirm"
        );
        ACCOUNT.with(|account| account.set(0));
        PREFIX.with(|prefix| *prefix.borrow_mut() = None);
    }
}
