import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createAssociatedTokenAccount, createMint, getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import { AmmTest } from "../target/types/amm_test";

describe("amm-test", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AmmTest as Program<AmmTest>;

  const owner: Keypair = anchor.web3.Keypair.generate();

  it("request sol", async () => {
    // request 10 sol
    const response = await provider.connection.requestAirdrop(
      owner.publicKey,
      LAMPORTS_PER_SOL * 10
    );

    // ide told it is deprecated but theres no way use TransactionConfirmationConfig. so use this.
    await provider.connection.confirmTransaction(response, "confirmed");
    // check the balance
    const balance = await provider.connection.getBalance(owner.publicKey);
    expect(balance).to.equal(LAMPORTS_PER_SOL * 10);

  });

  it("Is initialized!", async () => {


    //create mint
    const mint_a = await createMint(provider.connection, owner, owner.publicKey, owner.publicKey, 9, undefined, { "commitment": "confirmed" }, TOKEN_PROGRAM_ID);

    let owner_ata = await createAssociatedTokenAccount(provider.connection, owner, mint_a, owner.publicKey, { "commitment": "confirmed" }, TOKEN_PROGRAM_ID);

    let owner_ata_account = await getAccount(provider.connection, owner_ata, "confirmed");

    // get sol balance of owner of mint a ata
    const bal = await provider.connection.getBalance(owner_ata);
    console.log("sol balance : " + bal);
    console.log("owner: " + owner.publicKey.toString());
    console.log(owner_ata_account.amount);
    console.log(owner_ata_account.owner.toString());


    // Add your test here.
    // const tx = await program.methods.initialize().accounts({}).signers([owner])

  });
});
