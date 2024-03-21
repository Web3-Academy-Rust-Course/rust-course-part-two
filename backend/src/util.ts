import { ApiPromise } from "@polkadot/api";

interface ResultDto {
    status: boolean;
    message: string;
}

const wait = async (seconds: number): Promise<void> => {
    await new Promise<void>((resolve) => {
        setTimeout(() => {
            resolve();
        }, seconds * 1000);
    });
};

const checkExtrinsicStatus = async (
    api: ApiPromise,
    pallet: string,
    extrinsicCall: string,
    userAddress: string
): Promise<ResultDto> => {
    await wait(6);
    const currentBlock = await api.rpc.chain.getBlock();
    const currentBlockNumber = currentBlock.block.header.number.toNumber();
    const currentBlockHash = await api.rpc.chain.getBlockHash(
        currentBlockNumber
    );

    const apiAt = await api.at(currentBlockHash);

    const events: any = await apiAt.query.system.events();
    const extrinsics: any = currentBlock.block.extrinsics;

    let message: string = "Unknown error";
    let status: boolean = false;

    // Map between the extrinsics and events
    extrinsics.forEach(({ method: { method, section } }: any, index: any) => {
        events
            // Filter out only events that cover the desired pallet and extrinsic call
            .filter(() => section === pallet && method === extrinsicCall)
            // Filter the specific events based on the phase and then the
            // index of our extrinsic in the block
            .filter(
                ({ phase }: any) =>
                    phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
            )
            // Check the events against the specific types we are looking for
            .forEach(({ event }: any) => {
                if (extrinsics[index].signer.toString() === userAddress) {
                    if (api.events.system.ExtrinsicSuccess.is(event)) {
                        // Extract the data for this event
                        const [dispatchInfo] = event.data;

                        status = true;
                        message = JSON.stringify(dispatchInfo.toHuman());
                    } else if (api.events.system.ExtrinsicFailed.is(event)) {
                        const [dispatchError, _]: any = event.data;
                        let errorInfo;

                        // Decode the error
                        if (dispatchError.isModule) {
                            // For module errors, we have the section indexed, lookup
                            // (For specific known errors, we can also do a check against the
                            // api.errors.<module>.<ErrorName>.is(dispatchError.asModule) guard)
                            const decoded = api.registry.findMetaError(
                                dispatchError.asModule
                            );

                            errorInfo = `${decoded.section}.${decoded.name}`;
                        } else {
                            // Other, CannotLookup, BadOrigin, no extra info
                            errorInfo = dispatchError.toString();
                        }

                        status = false;
                        message = errorInfo;
                    }
                }
            });
    });

    return {
        status: status,
        message: message,
    } as ResultDto;
};

export { checkExtrinsicStatus, wait, ResultDto };
