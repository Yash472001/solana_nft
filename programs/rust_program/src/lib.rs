// use merkle_distributor::merkle_proof::verify;
use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token;
use anchor_spl::token::{MintTo, Token};
use mpl_token_metadata::state::Creator;
use solana_program::native_token::LAMPORTS_PER_SOL;
use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2};


declare_id!("EAGn68tRZY54VV7HiJrkK4E172v7TJGpwsAJwcwcRHNa");

const RECEIVER: Pubkey = solana_program::pubkey!("GwLPwf7zLxyDEotinzBEpsy1krdv165AtpkGfAmg3fVP");

const OWNER :Pubkey = solana_program::pubkey!("7EoAnBURZAxymps28xXVW2cVLQ4JwsxdxXi5HR4YcdPK");



pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.into_iter() {
        if computed_hash <= proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&computed_hash, &proof_element]).0;
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&proof_element, &computed_hash]).0;
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
}

fn print_type_of<T>(_: &T) {
    msg!("{}", std::any::type_name::<T>())
}


#[program]
pub mod rust_program {
    use super::*;

    const root : [u8; 32] = [
            108, 152,  60, 145, 210,  10,  76, 53,
            252, 245, 206, 139, 156, 222, 102, 22,
            245, 181, 197, 211, 206,  36, 187, 39,
            208,  44, 100,  41,   1, 226, 253, 47
        ];

    const whitelist_amount:u64 = 5 * (LAMPORTS_PER_SOL/10);

    pub fn initialize(ctx: Context<Tokenid>) ->  Result<()> {

        if OWNER !=  ctx.accounts.user.key(){
            return Err(error!(ErrorCode::Onlyowner));
        }

        let base_account = &mut ctx.accounts.base_account;
        msg!("{:?}",base_account.count);
        base_account.authority = ctx.accounts.user.key();
        msg!("{:?}", base_account.authority);
        Ok(())

    }

    pub fn set_base_uri(ctx:Context<Baseuri>,uri: String) -> Result<()> {
        let base_account = &mut ctx.accounts.base_account;
        base_account.baseuri = uri;
        Ok(())
    }

    // pub fn batch_mint(
    //     ctx: Context<MintNFT>,
    //     title: String,
    //     proof: Vec<[u8; 32]>
    // ) -> Result<()> {
    //     let mut arr = vec![];
    //     for i in 

    // }
    


    // One function for whitelist and publicSale
    pub fn mint_nft(
        ctx: Context<MintNFT>,
        title: String,
        proof: Vec<[u8; 32]>
    ) -> Result<()> {

        let mut amount:u64 = LAMPORTS_PER_SOL;

        let base_account = &mut ctx.accounts.base_account;

        let mut uril = base_account.baseuri.to_string();

        let count = base_account.count.to_string();

        let extension = ".json".to_string();

        msg!("Initial : {:?}",base_account.count);
        uril.push_str("/");
        uril.push_str(&count);
        uril.push_str(&extension);
        
        msg!("{:?}",uril.clone());
        let leaf = anchor_lang::solana_program::keccak::hash(&ctx.accounts.payer.key().to_string().as_bytes());

        let whiteListed :bool = verify(proof,root,leaf.0);

        msg!("verify : {:?}",whiteListed);

        let base_account = &mut ctx.accounts.base_account;
        // msg!("{}",base_account);

        if whiteListed {
            msg!("WhiteListed Happend");
            amount = whitelist_amount;
        }

        if ctx.accounts.payer.lamports() < amount {
            return Err(error!(ErrorCode::NotEnoughSOL));
        }

        if ctx.accounts.recevier.key().clone() != RECEIVER {
            return Err(error!(ErrorCode::InvalidReceiver));
        }

        // for transfer sol
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &ctx.accounts.recevier.key(),
            amount 
        );
        
        let result = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.recevier.to_account_info()
            ],
        );

        // transfer sucessful then go further
        if result != Ok(()) {
            return Err(error!(ErrorCode::Mintpricetransferfailed));
        }
    
        //CPI TO TOKENPROGRAM FOR MINT SPL-TOKEN
            msg!("Initializing Mint NFT");
            
            let cpi_accounts = MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            };

            msg!("CPI Accounts Assigned");
            let cpi_program = ctx.accounts.token_program.to_account_info();
            msg!("CPI Program Assigned");
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            msg!("CPI Context Assigned");
            token::mint_to(cpi_ctx, 1)?;
            msg!("Token Minted !!!");

        
        let account_info = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];
        msg!("Account Info Assigned");

        // creator array
        let mut creator =  vec![
            mpl_token_metadata::state::Creator {
                address:RECEIVER,
                verified: false,
                share: 100,
            }
        ];

        //if RECEIVER of sol is the same as mint_authority then this is not ad into creator array
        if ctx.accounts.mint_authority.key().clone() != RECEIVER {
            creator.push(
                mpl_token_metadata::state::Creator {
                    address: ctx.accounts.mint_authority.key(),
                    verified: true,
                    share: 0,
                });
        };
        
        msg!("Creator Assigned");
        let symbol = std::string::ToString::to_string("symb");

        //CPI TO METAPLEX PROGRAM FOR CREATING METADATA ACCOUNT
        invoke(
            &create_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.payer.key(),
                title.clone(),
                symbol,
                uril.to_string().clone(),
                Some(creator),
                1000,
                true,
                false,
                None,
                None,
            ),
            account_info.as_slice(),
        )?;
        msg!("Metadata Account Created !!!");
        msg!("{:?}",ctx.accounts.metadata.data.borrow());


        let master_edition_infos = vec![
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];
        msg!("Master Edition Account Infos Assigned");

        //CPI TO METAPLEX PROGRAM FOR CREATING MASTER_EDITION
        invoke(
            &create_master_edition_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.master_edition.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                Some(0),
            ),
            master_edition_infos.as_slice(),
        )?;
        msg!("data : {:?}",ctx.accounts.metadata.key());
        msg!("Master Edition Nft Minted !!!");
        msg!("Pre  : {:?}",base_account.count);
        base_account.count += 1;
        msg!("Post : {:?}",base_account.count);
        Ok(())

   }
    
}

// MintNft Struct for Context
#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub mint_authority: Signer<'info>,
/// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub recevier:UncheckedAccount<'info>,
    // #[account(mut)]
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    // storage of baseuri and Token Id
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
}

#[derive(Accounts)]
pub struct Tokenid<'info> {
    //space for account + authority + count + baseuri
    #[account(init_if_needed, payer = user, space = 16 + 32 + 16 + 128)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program <'info, System>
}

#[derive(Accounts)]
pub struct Baseuri<'info> {
    // it checks authority and initialize authority if not than revert an error 
    #[account(mut,has_one = authority)]
    pub base_account: Account<'info, BaseAccount>,
    pub authority:Signer<'info>
}

//store data like baseUri ,TokenId
#[account]
pub struct BaseAccount {
    pub authority:Pubkey,
    pub count: u64,
    pub baseuri: String,
}

//custom error
#[error_code]
pub enum ErrorCode {
    #[msg("Not enough SOL to pay for this minting")]
    NotEnoughSOL,
    #[msg("Receiver is not matched")]
    InvalidReceiver,
    #[msg("Lamport transfer Failed")]
    Mintpricetransferfailed,
    #[msg("Not whitelisted User")]
    WhiteListedUser,
    #[msg("Yoy are not Owner")]
    Onlyowner
}
