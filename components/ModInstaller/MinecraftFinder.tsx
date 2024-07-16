import {useQuery} from "react-query";
import {dataDir, homeDir} from "@tauri-apps/api/path";
import {readDir,exists} from "@tauri-apps/api/fs";
import ModsFolder from "./ModsFolder.tsx";

export default function MinecraftFinder({osType}:{osType:string}){
    const pathInfo = useQuery("defaultPath",async () => {
        let defaultPath = "";
        switch(osType){
            case('Linux'):
                defaultPath = await homeDir()
                break

            case('Windows_NT'):
                defaultPath = await dataDir()
                break
            default:
                defaultPath = await dataDir()
                break
        }
        let readPath = await readDir(defaultPath)
        let minecraftExists = await exists(defaultPath+"/.minecraft")
        return({defaultPath,readPath,minecraftExists})
    })

    return(
        <>
            {pathInfo.isLoading?
                <>Loading...</>
                :
                pathInfo.isError?
                    <>Something went wrong!</>
                    :

                    <>
                        Minecraft file found at {pathInfo.data.defaultPath}
                        <ModsFolder path={pathInfo.data.defaultPath}/>
                    </>
            }
        </>
    )
}