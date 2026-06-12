#![no_std]
// Đã xóa `symbol_short` khỏi dòng use
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Symbol, Vec};

// ===== 1. DANH SÁCH LỖI (ERRORS) =====
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SplitPayError {
    AlreadyInitialized = 1,
    EmptyMembers = 2,
    TooManyMembers = 3,
    InvalidShares = 4,
    ContractPaused = 5,
    ZeroAmount = 6,
    Overflow = 7,
}

// ===== 2. CẤU TRÚC DỮ LIỆU TỐI ƯU =====
#[derive(Clone)] // <--- ĐÃ THÊM DÒNG NÀY ĐỂ SỬA LỖI VÒNG LẶP
#[contracttype]
pub struct Shareholder {
    pub account: Address,
    pub share: u32, // Tỷ lệ: 10000 = 100%
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Shareholders,
    TokenAddress,
    TotalReceived,
    TotalDistributed,
    Remainder,
    IsPaused,
}

// ===== 3. KHAI BÁO HỢP ĐỒNG =====
#[contract]
pub struct SplitPayContract;

#[contractimpl]
impl SplitPayContract {
    
    // --- 3.1. HÀM KHỞI TẠO ---
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        shareholders: Vec<Shareholder>,
    ) -> Result<(), SplitPayError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SplitPayError::AlreadyInitialized);
        }
        if shareholders.is_empty() {
            return Err(SplitPayError::EmptyMembers);
        }
        if shareholders.len() > 20 {
            return Err(SplitPayError::TooManyMembers);
        }

        let mut total_shares: u32 = 0;
        for sh in shareholders.iter() {
            total_shares += sh.share;
        }
        if total_shares != 10_000u32 {
            return Err(SplitPayError::InvalidShares);
        }

        let storage = env.storage().instance();
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::TokenAddress, &token);
        storage.set(&DataKey::Shareholders, &shareholders);
        storage.set(&DataKey::TotalReceived, &0i128);
        storage.set(&DataKey::TotalDistributed, &0i128);
        storage.set(&DataKey::Remainder, &0i128);
        storage.set(&DataKey::IsPaused, &false);

        // Emit sự kiện
        env.events().publish((Symbol::new(&env, "initialized"),), (admin, token, shareholders.len()));
        Ok(())
    }

    // --- 3.2. HÀM CỐT LÕI: THANH TOÁN VÀ CHIA TIỀN ---
    pub fn deposit_and_split(env: Env, from: Address, amount: i128) -> Result<(), SplitPayError> {
        from.require_auth();

        let is_paused: bool = env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false);
        if is_paused {
            return Err(SplitPayError::ContractPaused);
        }
        if amount <= 0 {
            return Err(SplitPayError::ZeroAmount);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::TokenAddress).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        let shareholders: Vec<Shareholder> = env.storage().instance().get(&DataKey::Shareholders).unwrap();
        let mut total_distributed: i128 = 0;

        for sh in shareholders.iter() {
            let member_amount = amount.checked_mul(sh.share as i128).ok_or(SplitPayError::Overflow)? / 10_000i128;
            if member_amount > 0 {
                token_client.transfer(&env.current_contract_address(), &sh.account, &member_amount);
                total_distributed = total_distributed.checked_add(member_amount).ok_or(SplitPayError::Overflow)?;
            }
        }

        let remainder = amount - total_distributed;
        let prev_remainder: i128 = env.storage().instance().get(&DataKey::Remainder).unwrap_or(0);
        env.storage().instance().set(&DataKey::Remainder, &(prev_remainder + remainder));

        let prev_received: i128 = env.storage().instance().get(&DataKey::TotalReceived).unwrap_or(0);
        let prev_distributed: i128 = env.storage().instance().get(&DataKey::TotalDistributed).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalReceived, &(prev_received + amount));
        env.storage().instance().set(&DataKey::TotalDistributed, &(prev_distributed + total_distributed));

        env.events().publish((Symbol::new(&env, "split"),), (from, amount, total_distributed, remainder));
        Ok(())
    }

    // --- 3.3. HÀM CẬP NHẬT CỔ ĐÔNG ---
    pub fn update_shares(env: Env, new_shareholders: Vec<Shareholder>) -> Result<(), SplitPayError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if new_shareholders.is_empty() || new_shareholders.len() > 20 {
            return Err(SplitPayError::EmptyMembers);
        }
        let mut total: u32 = 0;
        for sh in new_shareholders.iter() {
            total += sh.share;
        }
        if total != 10_000u32 {
            return Err(SplitPayError::InvalidShares);
        }

        env.storage().instance().set(&DataKey::Shareholders, &new_shareholders);
        env.events().publish((Symbol::new(&env, "shares_updated"),), (admin, new_shareholders.len()));
        Ok(())
    }

    // --- 3.4. HÀM ADMIN: RÚT TIỀN LẺ ---
    pub fn claim_remainder(env: Env) -> Result<i128, SplitPayError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let remainder: i128 = env.storage().instance().get(&DataKey::Remainder).unwrap_or(0);
        if remainder > 0 {
            let token_addr: Address = env.storage().instance().get(&DataKey::TokenAddress).unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&env.current_contract_address(), &admin, &remainder);
            env.storage().instance().set(&DataKey::Remainder, &0i128);
        }
        Ok(remainder)
    }

    // --- 3.5. HÀM ADMIN: TẠM DỪNG ---
    pub fn set_paused(env: Env, paused: bool) -> Result<(), SplitPayError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::IsPaused, &paused);
        Ok(())
    }

    // --- 3.6. CÁC HÀM XEM DỮ LIỆU (VIEW FUNCTIONS) ---
    pub fn get_shareholders(env: Env) -> Vec<Shareholder> {
        env.storage().instance().get(&DataKey::Shareholders).unwrap()
    }

    pub fn get_stats(env: Env) -> (i128, i128, i128) {
        let r = env.storage().instance().get(&DataKey::TotalReceived).unwrap_or(0);
        let d = env.storage().instance().get(&DataKey::TotalDistributed).unwrap_or(0);
        let rem = env.storage().instance().get(&DataKey::Remainder).unwrap_or(0);
        (r, d, rem)
    }
}