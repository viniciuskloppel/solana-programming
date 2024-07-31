import bs58 from "bs58";
import keypair from "./dev-wallet.json";
import fs from "node:fs";

const byteArray = bs58.encode(keypair);
fs.writeFileSync(
  "./convert-result-dev-wallet.json",
  `[${byteArray.toString()}]`
);
