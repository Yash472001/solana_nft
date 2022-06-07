const anchor = require("@project-serum/anchor");
const {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  MINT_SIZE,
} = require("@solana/spl-token");

const { PublicKey, SystemProgram, LAMPORTS_PER_SOL } = anchor.web3;

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const receiver = new PublicKey("GwLPwf7zLxyDEotinzBEpsy1krdv165AtpkGfAmg3fVP");

anchor.setProvider(anchor.Provider.local("https://api.devnet.solana.com"));
const idl = JSON.parse(
  require("fs").readFileSync("./target/idl/rust_program.json")
);
const programId = new PublicKey(
  "EAGn68tRZY54VV7HiJrkK4E172v7TJGpwsAJwcwcRHNa"
);
const program = new anchor.Program(idl, programId);
const mintKey = anchor.web3.Keypair.generate();

const getMetadata = async (mint) => {
  return (
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  )[0];
};

const getMasterEdition = async (mint) => {
  return (
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  )[0];
};


async function intermediate() {

  const lamports =
  await program.provider.connection.getMinimumBalanceForRentExemption(
    MINT_SIZE
  );

  const NftTokenAccount = await getAssociatedTokenAddress(
    mintKey.publicKey,
    program.provider.wallet.publicKey
  );
  console.log("NFT Account: ", NftTokenAccount.toBase58());

  const mint_tx = new anchor.web3.Transaction().add(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: program.provider.wallet.publicKey,
      newAccountPubkey: mintKey.publicKey,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID,
      lamports,
    }),
    createInitializeMintInstruction(
      mintKey.publicKey,
      0,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey
    ),
    createAssociatedTokenAccountInstruction(
      program.provider.wallet.publicKey,
      NftTokenAccount,
      program.provider.wallet.publicKey,
      mintKey.publicKey
    )
  );


  console.log(
    await program.provider.connection.getParsedAccountInfo(mintKey.publicKey)
  );
  // console.log("Account: ", res);
  console.log("Mint key: ", mintKey.publicKey.toString());
  console.log("User: ", program.provider.wallet.publicKey.toString());
  const metadataAddress = await getMetadata(mintKey.publicKey);
  const masterEdition = await getMasterEdition(mintKey.publicKey);
  console.log("Metadata address: ", metadataAddress.toBase58());
  console.log("MasterEdition: ", masterEdition.toBase58());


  


  const tx = await program.instruction.mintNftPrice(
    mintKey.publicKey,
    "https://gateway.pinata.cloud/ipfs/QmQEY88g33yzSmVBonLAY8tK6cgY8F4N8yqxUEYrdS3W99",
    "NFT ",
    {
      accounts: {
        mintAuthority: program.provider.wallet.publicKey,
        mint: mintKey.publicKey,
        recevier:receiver,
        tokenAccount: NftTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        payer: program.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        masterEdition: masterEdition,
      },
    }
  );
  console.log("Your transaction signature", tx);

  mint_tx.add(tx);

  const res = await program.provider.send(mint_tx, [mintKey]);


}


intermediate().then(() =>console.log("done"));


