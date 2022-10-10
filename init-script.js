
import * as anchor from "@project-serum/anchor";

const args = require('minimist')(process.argv.slice(2));


const provider = anchor.AnchorProvider().env();
anchor.setProvider(provider);

const CHAINLINK_PROGRAM_ID = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
const DIVISOR = 100000000;

// Data feed account address
// Default is SOL / USD
const default_feed = "HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6";
const CHAINLINK_FEED = args['feed'] || default_feed;


async function main() {
    const idl = JSON.parse(require('fs').readFileSync("./target/idl/vega_core.json", "utf-8"));

    // Address of the deployed program.
    const programId = new anchor.web3.PublicKey(args['program']);

    const program = new anchor.Program(idl, programId);

    console.log(program.programId);

}

main().then(() => "done");

// anchor deploy --provider.cluster devnet
// anchor idl init --provider.cluster devnet  -f target/idl/vega_core.json `solana address -k target/deploy/vega_core-keypair.json`
// Please note that everytime you redeploy your solana program,
// you need to tell solana how your program api looks like, we can do so with anchor idl upgrade instead of anchor idl init