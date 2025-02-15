use std::ops::{Deref, DerefMut};

use crate::{prelude::AnchorDeserialize, ToAccountInfos, ToAccountMetas};
use crate::{Owner, ToAccountInfo};
use otter_solana_program::{
    account_info::AccountInfo, error::Error, instruction::AccountMeta, pubkey::Pubkey, Key, Result,
};

#[derive(Debug)]
pub struct Account<'info, T> {
    pub account: T,
    pub info: AccountInfo<'info>,
}

impl<'a, T> Account<'a, T> {
    fn new(info: AccountInfo<'a>, account: T) -> Account<'a, T> {
        Self { info, account }
    }

    pub fn reload(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn into_inner(self) -> T {
        self.account
    }

    pub fn set_inner(&mut self, inner: T) {
        self.account = inner;
    }
}

impl<'a, T: AnchorDeserialize + Owner> Account<'a, T> {
    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'a>) -> Result<Account<'a, T>> {
        if
        /* info.owner == &system_program::ID && */
        info.lamports() == 0 {
            // return Err(ErrorCode::AccountNotInitialized.into());
            return Err(Error);
        }
        if info.owner != &T::owner() {
            return Err(Error);
        }
        let mut data: &[u8] = info.try_borrow_data()?;
        Ok(Account::new(
            info.clone(),
            T::deserialize(&mut data).map_err(|_| Error)?,
        ))
    }

    #[inline(never)]
    pub fn try_from_unchecked(info: &AccountInfo<'a>) -> Result<Account<'a, T>> {
        Self::try_from(info)
    }
}

impl<'info, T> ToAccountMetas for Account<'info, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T> ToAccountInfo<'info> for Account<'info, T> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.info.clone()
    }
}

impl<'info, T> ToAccountInfos<'info> for Account<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'a, T> Deref for Account<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'a, T> AsRef<T> for Account<'a, T>
where
    <Account<'a, T> as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> DerefMut for Account<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(feature = "anchor-debug")]
        if !self.info.is_writable {
            solana_program::msg!("The given Account is not mutable");
            panic!();
        }
        &mut self.account
    }
}

impl<'info, T> Key for Account<'info, T> {
    fn key(&self) -> Pubkey {
        *self.info.key
    }
}

#[cfg(any(kani, feature = "kani"))]
impl<'info, T> kani::Arbitrary for Account<'info, T>
where
    T: kani::Arbitrary,
{
    fn any() -> Self {
        Self {
            info: kani::any(),
            account: kani::any(),
        }
    }
}
