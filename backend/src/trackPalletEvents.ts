import { ApiPromise, HttpProvider } from "@polkadot/api";
import { wait } from "./util";

const main = async () => {
    const provider = new HttpProvider("http://127.0.0.1:9944");
    const api = await new ApiPromise({ provider }).isReady;

    while (true) {
        const currentBlock = await api.rpc.chain.getBlock();
        const currentBlockNumber = currentBlock.block.header.number.toNumber();
        const currentBlockHash = await api.rpc.chain.getBlockHash(
            currentBlockNumber
        );

        const apiAt = await api.at(currentBlockHash);

        const events: any = await apiAt.query.system.events();

        events
            // .filter((event: any) => event.event.section === "defi")
            .forEach((filteredEvent: any) => {
                console.log(filteredEvent.toHuman());
            });

        await wait(6);
    }
};

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(() => {
        process.exit(0);
    });
