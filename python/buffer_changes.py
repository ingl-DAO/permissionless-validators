import os
from solana.publickey import PublicKey
import sys
from src.state import keypair_from_json

upgraded_program_keypair = keypair_from_json("./deploy/keypair.json")
pda_authority_key = PublicKey.find_program_address([b"authority", bytes(upgraded_program_keypair.public_key)], upgraded_program_keypair.public_key)[0]
command = f"solana program set-buffer-authority ./deploy/buffer.json --new-buffer-authority {pda_authority_key} -u {sys.argv[1]}"
# print(command)
os.system(command)