use crate::{
    colored_log,
    state::LogColors::{Blank, Blue, Green, Red},
};
use ingl_macros::InglErr;
use solana_program::program_error::ProgramError;

const LOG_LEVEL: u8 = 5;
#[derive(InglErr, Debug)]
pub enum InglError {
    #[err("Provided address is dissimilar from the expected one")]
    AddressMismatch, //0

    #[err("Provided Struct Type does not match expected value.")]
    InvalidStructType, //1

    #[err("Funds are Not located in the appropriate pool for this instruction.")]
    InvalidFundsLocation, //2

    #[err("Executing a process earlier than is allowed.")]
    TooEarly, //3

    #[err("Executing a process later than is allowed.")]
    TooLate, //4

    #[err("A vote had already occured with the specified accounts.")]
    AlreadyVoted, //5

    #[err("A certain operation yielded a value beyond bounds.")]
    BeyondBounds, //6

    #[err("Validation Phrase Found in the sent account is different from that expected.")]
    InvalidValPhrase, //7

    #[err("History Feed Price Can't Be zero.")]
    ZeroPrice, //8

    #[err("The account type must be a buffer, a delineation exists between the sent type and the expected type.")]
    ExpectedBufferAccount, //9

    #[err("An Error Occured while unwrapping an Option")]
    OptionUnwrapError, //10

    #[err("Failed to verify the History buffer keys sent")]
    InvalidHistoryBufferKeys, //11

    #[err("Failed to verify the Config data")]
    InvalidConfigData, //12

    #[err("Failed to verify the Uris data")]
    InvalidUrisAccountData, //13

    #[err("Invalid Data")]
    InvalidData, //14

    #[err("Cannot Verify NFT Ownership")]
    NFTBalanceCheckError, //15
}
