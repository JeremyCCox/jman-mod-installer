import {QueryClient, QueryClientProvider, useQuery} from "react-query";
import {type} from "@tauri-apps/api/os";
import MinecraftFinder from "./MinecraftFinder.tsx";

export default function InstallerBase(){
    const osInfo = useQuery('osType',async () => {
        return (await type())
    })
    return(
        <>
            {osInfo.isLoading?
                <></>
                :
                osInfo.error?
                    <>Could not verify operating system!</>
                    :
                    <div className={'m-16 relative'}>
                        <MinecraftFinder osType={osInfo.data||"Windows_NT"}/>
                    </div>

            }
        </>
    )
}