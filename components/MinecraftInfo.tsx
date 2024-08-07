import {useQuery} from "react-query";
import {type} from "@tauri-apps/api/os";
import MinecraftFinder from "./ModInstaller/MinecraftFinder";

export default function MinecraftInfo(){
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
                    <>
                        <br/>
                        <MinecraftFinder osType={osInfo.data||"Windows_NT"}/>

                    </>

            }
        </>
    )
}