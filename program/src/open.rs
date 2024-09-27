use ore_boost_api::{
    consts::STAKE,
    instruction::Open,
    state::{Boost, Stake},
};
use solana_program::system_program;
use steel::*;

/// Open creates a new stake account.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Open::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, payer_info, boost_info, mint_info, stake_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    payer_info.is_signer()?;
    boost_info
        .to_account::<Boost>(&ore_boost_api::ID)?
        .check(|b| b.mint == *mint_info.key)?;
    stake_info.is_empty()?.is_writable()?.has_seeds(
        &[STAKE, signer_info.key.as_ref(), boost_info.key.as_ref()],
        args.stake_bump,
        &ore_boost_api::id(),
    )?;
    mint_info.to_mint()?;
    system_program.is_program(&system_program::ID)?;

    // Get clock
    let clock = Clock::get().unwrap();

    // Initialize the stake account.
    create_account::<Stake>(
        stake_info,
        &ore_boost_api::id(),
        &[
            STAKE,
            signer_info.key.as_ref(),
            boost_info.key.as_ref(),
            &[args.stake_bump],
        ],
        system_program,
        payer_info,
    )?;
    let stake = stake_info.to_account_mut::<Stake>(&ore_boost_api::ID)?;
    stake.authority = *signer_info.key;
    stake.balance = 0;
    stake.boost = *boost_info.key;
    stake.last_stake_at = clock.unix_timestamp;

    Ok(())
}
