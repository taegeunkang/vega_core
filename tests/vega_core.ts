import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  getMinimumBalanceForRentExemptAccount,
  getMinimumBalanceForRentExemptAccountWithExtensions,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { BN, min } from "bn.js";
import { expect } from "chai";
import { VegaCore } from "../target/types/vega_core";

describe("vega_core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VegaCore as Program<VegaCore>;

  const owner: Keypair = anchor.web3.Keypair.generate();
  let mint: PublicKey;
  let lp_mint: PublicKey;
  let owner_ata: PublicKey;
  let owner_lp_ata: PublicKey;
  let config_pda: PublicKey;
  let pool_pda: PublicKey;
  let pool_vault_ata: PublicKey;
  let pool_lp_vault_ata: PublicKey;
  before(async () => { });

  const request_sol = async (user: PublicKey) => {
    // request 10 sol
    const response = await provider.connection.requestAirdrop(
      user,
      LAMPORTS_PER_SOL * 10
    );

    // ide told it is deprecated but theres no way use TransactionConfirmationConfig. so use this.
    await provider.connection.confirmTransaction(response, "confirmed");
  };

  it("aridrop sol", async () => {
    // request 10 sol
    await request_sol(owner.publicKey);
    // check the balance
    const balance = await provider.connection.getBalance(owner.publicKey);
    expect(balance).to.equal(LAMPORTS_PER_SOL * 10);
  });
  it("create tokens and mint ", async () => {
    //create mint
    mint = await createMint(
      provider.connection,
      owner,
      owner.publicKey,
      owner.publicKey,
      9,
      undefined,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );
    lp_mint = await createMint(
      provider.connection,
      owner,
      owner.publicKey,
      owner.publicKey,
      9,
      undefined,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    owner_ata = await createAssociatedTokenAccount(
      provider.connection,
      owner,
      mint,
      owner.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );
    owner_lp_ata = await createAssociatedTokenAccount(
      provider.connection,
      owner,
      lp_mint,
      owner.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    await mintTo(
      provider.connection,
      owner,
      mint,
      owner_ata,
      owner,
      100000000 * LAMPORTS_PER_SOL,
      [],
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );
    await mintTo(
      provider.connection,
      owner,
      lp_mint,
      owner_lp_ata,
      owner,
      100000000 * LAMPORTS_PER_SOL,
      [],
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );
    let owner_ata_account = await getAccount(
      provider.connection,
      owner_ata,
      "confirmed",
      TOKEN_PROGRAM_ID
    );

    expect(owner_ata_account.amount.toString()).to.equal(
      (100000000 * LAMPORTS_PER_SOL).toString()
    );
  });

  it("initialize", async () => {
    const [config_pda, _] = findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode("config"))],
      program.programId
    );
    await program.methods
      .initialize()
      .accounts({
        signer: owner.publicKey,
        config: config_pda,
        lpMint: lp_mint,
      })
      .signers([owner])
      .rpc();
    const [config_acc] = await program.account.config.all();
    expect(owner.publicKey.toString()).to.equal(
      config_acc.account.authority.toString()
    );
  });

  it("create pool", async () => {
    [config_pda] = findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode("config"))],
      program.programId
    );
    [pool_pda] = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("pool")),
        config_pda.toBuffer(),
        mint.toBuffer(),
      ],
      program.programId
    );
    [pool_vault_ata] = findProgramAddressSync(
      [pool_pda.toBuffer(), mint.toBuffer()],
      program.programId
    );
    [pool_lp_vault_ata] = findProgramAddressSync(
      [pool_pda.toBuffer(), lp_mint.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .createPool()
      .accounts({
        signer: owner.publicKey,
        pool: pool_pda,
        mint: mint,
        lpMint: lp_mint,
        poolVault: pool_vault_ata,
        poolLpVault: pool_lp_vault_ata,
        userLpAta: owner_lp_ata,
        userAta: owner_ata,
        config: config_pda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    await provider.connection.confirmTransaction(tx, "confirmed");

    let mint_amount = (await getAccount(provider.connection, owner_ata, "confirmed", TOKEN_PROGRAM_ID)).amount;
    let lp_amount = (await getAccount(provider.connection, owner_lp_ata, "confirmed", TOKEN_PROGRAM_ID)).amount;

    expect(mint_amount.toString()).to.equal((90000000 * LAMPORTS_PER_SOL).toString());
    expect(lp_amount.toString()).to.equal('0');

  });
  it("buy mint", async () => {

    let amount_in_sol = 1 * LAMPORTS_PER_SOL;
    const tx = await program.methods.buy(new BN(amount_in_sol)).accounts({ signer: owner.publicKey, mint: mint, userAta: owner_ata, pool: pool_pda, poolVault: pool_vault_ata, config: config_pda, tokenProgram: TOKEN_PROGRAM_ID }).signers([owner]).rpc();
    await provider.connection.confirmTransaction(tx, "confirmed");

    let vega_amount = (await getAccount(provider.connection, owner_ata, "confirmed", TOKEN_PROGRAM_ID)).amount;
    let sol_amount = await provider.connection.getBalance(pool_pda, { commitment: "confirmed" });
    let exempt = await getMinimumBalanceForRentExemptAccount(provider.connection, "confirmed");
    expect(sol_amount).to.greaterThan(1 * LAMPORTS_PER_SOL);
    expect(vega_amount.toString()).to.equal(((90000010 * LAMPORTS_PER_SOL).toString()));

  });
  it("sell mint", async () => {
    let amount_in_mint = 10 * LAMPORTS_PER_SOL;

    let before_sol = await provider.connection.getBalance(owner.publicKey, { commitment: "confirmed" });
    const tx = await program.methods.sell(new BN(amount_in_mint)).accounts({ signer: owner.publicKey, mint: mint, userAta: owner_ata, pool: pool_pda, poolVault: pool_vault_ata, config: config_pda, tokenProgram: TOKEN_PROGRAM_ID }).signers([owner]).rpc();
    await provider.connection.confirmTransaction(tx, "confirmed");
    let after_sol = await provider.connection.getBalance(owner.publicKey, { commitment: "confirmed" });
    expect(after_sol).to.equal(before_sol + 1 * LAMPORTS_PER_SOL);

  })


  it("deposit", async () => {
    let [owner_pool_info_pda, _4] = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("user_pool_info")),
        owner.publicKey.toBuffer(),
        pool_pda.toBuffer(),
      ],
      program.programId
    );
    try {
      const before_lp = await getAccount(
        provider.connection,
        owner_lp_ata,
        "confirmed",
        TOKEN_PROGRAM_ID
      );

      const tx = await program.methods
        .deposit(new anchor.BN(100 * LAMPORTS_PER_SOL))
        .accounts({
          signer: owner.publicKey,
          pool: pool_pda,
          poolVault: pool_vault_ata,
          poolLpVault: pool_lp_vault_ata,
          mint: mint,
          lpMint: lp_mint,
          userAta: owner_ata,
          userLpAta: owner_lp_ata,
          userPoolInfo: owner_pool_info_pda,
          config: config_pda,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([owner])
        .rpc();
      await provider.connection.confirmTransaction(tx, "confirmed");

      const after_lp = await getAccount(
        provider.connection,
        owner_lp_ata,
        "confirmed",
        TOKEN_PROGRAM_ID
      );

      expect(before_lp.amount).not.to.equal(after_lp.amount);
    } catch (error) {
      console.log(error);
    }
  });

  it("another user deposit and compare lp", async () => {
    let another_01 = Keypair.generate();

    let [another_01_pool_info_pda, _4] = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("user_pool_info")),
        another_01.publicKey.toBuffer(),
        pool_pda.toBuffer(),
      ],
      program.programId
    );

    // create other account 01
    await request_sol(another_01.publicKey);
    let another_01_mint_ata: PublicKey = await createAssociatedTokenAccount(
      provider.connection,
      another_01,
      mint,
      another_01.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    let another_01_lp_mint_ata: PublicKey = await createAssociatedTokenAccount(
      provider.connection,
      another_01,
      lp_mint,
      another_01.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    await mintTo(
      provider.connection,
      owner,
      mint,
      another_01_mint_ata,
      owner,
      101 * LAMPORTS_PER_SOL,
      undefined,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .deposit(new BN(100 * LAMPORTS_PER_SOL))
      .accounts({
        signer: another_01.publicKey,
        pool: pool_pda,
        poolVault: pool_vault_ata,
        poolLpVault: pool_lp_vault_ata,
        mint: mint,
        lpMint: lp_mint,
        userAta: another_01_mint_ata,
        userLpAta: another_01_lp_mint_ata,
        userPoolInfo: another_01_pool_info_pda,
        config: config_pda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([another_01])
      .rpc();

    await provider.connection.confirmTransaction(tx, "confirmed");

    const another_01_lp_mint_account = await getAccount(
      provider.connection,
      another_01_lp_mint_ata,
      "confirmed",
      TOKEN_PROGRAM_ID
    );
    const owner_lp_mint_account = await getAccount(
      provider.connection,
      owner_lp_ata,
      "confirmed",
      TOKEN_PROGRAM_ID
    );

    expect(another_01_lp_mint_account.amount.toString()).to.equal(
      owner_lp_mint_account.amount.toString()
    );
  });

  it("withdraw", async () => {

    let [owner_pool_info_pda, _4] = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("user_pool_info")),
        owner.publicKey.toBuffer(),
        pool_pda.toBuffer(),
      ],
      program.programId
    );


    const tx = await program.methods.withdraw().accounts({ signer: owner.publicKey, pool: pool_pda, poolVault: pool_vault_ata, poolLpVault: pool_lp_vault_ata, mint: mint, lpMint: lp_mint, userAta: owner_ata, userLpAta: owner_lp_ata, userPoolInfo: owner_pool_info_pda, config: config_pda, rent: anchor.web3.SYSVAR_RENT_PUBKEY, clock: anchor.web3.SYSVAR_CLOCK_PUBKEY, tokenProgram: TOKEN_PROGRAM_ID }).signers([owner]).rpc();

    await provider.connection.confirmTransaction(tx, "confirmed");

  });
});
