import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { BN, min } from "bn.js";
import { expect } from "chai";
import { VegaCore } from "../target/types/vega_core";

describe("amm-test", () => {
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
  before(async () => {});

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
        config: config_pda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    await provider.connection.confirmTransaction(tx, "confirmed");
  });
  it("add liquidity", async () => {
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
        .addLiquidity(new anchor.BN(100 * LAMPORTS_PER_SOL))
        .accounts({
          authority: owner.publicKey,
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

  it("other user add liquidity compare lp", async () => {
    let other_01 = Keypair.generate();

    let [other_01_pool_info_pda, _4] = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("user_pool_info")),
        other_01.publicKey.toBuffer(),
        pool_pda.toBuffer(),
      ],
      program.programId
    );

    // create other account 01
    await request_sol(other_01.publicKey);
    let other_01_mint_ata: PublicKey = await createAssociatedTokenAccount(
      provider.connection,
      other_01,
      mint,
      other_01.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    let other_01_lp_mint_ata: PublicKey = await createAssociatedTokenAccount(
      provider.connection,
      other_01,
      lp_mint,
      other_01.publicKey,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    await mintTo(
      provider.connection,
      owner,
      mint,
      other_01_mint_ata,
      owner,
      101 * LAMPORTS_PER_SOL,
      undefined,
      { commitment: "confirmed" },
      TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .addLiquidity(new BN(100 * LAMPORTS_PER_SOL))
      .accounts({
        authority: owner.publicKey,
        signer: other_01.publicKey,
        pool: pool_pda,
        poolVault: pool_vault_ata,
        poolLpVault: pool_lp_vault_ata,
        mint: mint,
        lpMint: lp_mint,
        userAta: other_01_mint_ata,
        userLpAta: other_01_lp_mint_ata,
        userPoolInfo: other_01_pool_info_pda,
        config: config_pda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([other_01])
      .rpc();

    await provider.connection.confirmTransaction(tx, "confirmed");

    const other_01_lp_mint_account = await getAccount(
      provider.connection,
      other_01_lp_mint_ata,
      "confirmed",
      TOKEN_PROGRAM_ID
    );
    const owner_lp_mint_account = await getAccount(
      provider.connection,
      owner_lp_ata,
      "confirmed",
      TOKEN_PROGRAM_ID
    );

    expect(other_01_lp_mint_account.amount.toString()).to.equal(
      owner_lp_mint_account.amount.toString()
    );
  });
});
