import {useQuery} from "react-query";
import {readDir} from "@tauri-apps/api/fs";
import DownloadMod from "./DownloadMod.tsx";

export default function ModsFolder({path}:{path:string}){
    const modsReadout= useQuery('modsFolder',async () => {
        let modsReadout = await readDir(`${path}.minecraft/mods`)
        return (modsReadout)
    })

    return(
        <>
            {
                modsReadout.isLoading?
                    <>Loading</>
                    :
                    modsReadout.isError?
                        <>Error</>
                        :
                        <>
                            <pre >
                                {JSON.stringify(modsReadout.data, null, 2)}
                            </pre>
                            <DownloadMod/>
                        </>
                        }
        </>
    )
}