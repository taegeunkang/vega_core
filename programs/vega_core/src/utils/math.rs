use std::ops::Mul;

pub fn sqrt(y: u64) -> u64 {
    let mut _z: u64 = 0;

    if y > 3 {
        _z = y;
        let mut x = y.checked_div(2).unwrap().checked_add(1).unwrap();

        loop {
            _z = x;
            x = y
                .checked_div(x)
                .unwrap()
                .checked_add(x)
                .unwrap()
                .checked_div(2)
                .unwrap();

            if x >= _z {
                break;
            }
        }
    } else if y != 0 {
        _z = 1;
    }

    _z
}

pub fn calc_fee_amount(_amount: u64, _fee_rate: u16) -> u64 {
    _amount
        .checked_mul(_fee_rate as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
}

pub fn calc_lp_amount(_pool_vault_amount: u64, _pool_lp_supply: u64, amount_in: u64) -> u64 {
    let pool_amount: u64 = _pool_vault_amount;

    let total_supply: u64 = _pool_lp_supply;
    let mut _liquidity: u64 = 0;

    if total_supply == 0 {
        _liquidity = sqrt(amount_in).checked_sub(1000).unwrap();
        //1000개 기본 락
    } else {
        _liquidity = amount_in
            .checked_mul(total_supply)
            .unwrap()
            .checked_div(pool_amount)
            .unwrap();
    }

    _liquidity
}

pub fn calc_reward_percent(
    _deposit_amount: u64,
    _start: u64,
    _end: u64,
    _before_lp: u64,
    _after_lp: u64,
) -> u64 {
    let duration: u64 = _end.checked_sub(_start).unwrap();
    let standard_time: u64 = 2;
    let reward_rate: u64 = 3;
    let reward_amount_per_2_sec: u64 = _deposit_amount
        .checked_mul(reward_rate)
        .unwrap()
        .checked_div(10_000_000)
        .unwrap();

    let mut reward: u64 = duration
        .checked_div(standard_time)
        .unwrap()
        .checked_mul(reward_amount_per_2_sec)
        .unwrap();

    let after: u64 = _after_lp.mul(100);

    let p = after.checked_div(_before_lp).unwrap();

    reward = reward.checked_mul(p).unwrap().checked_div(100).unwrap();
    reward
    // fee amount = amount * fee_rate (ex: 30) / 10_000 -> 0.3%
}

pub fn calc_pl(_entry: u64, _out: u64, _way: u8) -> (u64, u8) {
    let _r : u64;
    let pl : bool;
    if _way == 0 {
        if _entry <= _out {
            let a = _entry.checked_mul(100).unwrap();
            let b = _out.checked_sub(_entry).unwrap().checked_mul(100).unwrap();
            _r = b.checked_div(a).unwrap().checked_div(100).unwrap();
            pl = true;

        } else {
            let a = _entry.checked_mul(100).unwrap();
            let b = _entry.checked_sub(_out).unwrap().checked_mul(100).unwrap();
            _r = b.checked_div(a).unwrap().checked_div(100).unwrap();
            pl = false;
        }
    } else {

    }

    (_r, pl);
}
