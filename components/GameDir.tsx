import {useQuery} from "react-query";
import {type} from "@tauri-apps/api/os";
import {exists, FileEntry, readDir} from "@tauri-apps/api/fs";
import {dataDir, homeDir} from "@tauri-apps/api/path";

export default function GameDir(){
    const osInfo = useQuery('osType',async () => {
        return (await type())
    })
    const pathInfo = useQuery("defaultPath",async ():Promise<{defaultPath:string,readPath:FileEntry[],minecraftExists:boolean}> => {
        let defaultPath = "";
        switch(osInfo.data){
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
        let readPath = await readDir(defaultPath+".minecraft")
        let minecraftExists = await exists(defaultPath+".minecraft")
        return({defaultPath:defaultPath+".minecraft",readPath,minecraftExists})
    },{initialData:{defaultPath:"",readPath:[],minecraftExists:false},enabled:!!osInfo.data});
    return(
        <>
            {pathInfo.data?.defaultPath&&
                <input type={"text"} name={"defaultGameDir"} value={pathInfo.data.defaultPath}/>
            }
        </>
    )
}