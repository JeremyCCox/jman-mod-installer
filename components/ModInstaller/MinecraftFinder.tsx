import {useQuery} from "react-query";
import {dataDir, homeDir} from "@tauri-apps/api/path";
import {readDir, exists, FileEntry} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";
import ProfileDisplay from "../profiles/ProfileDisplay.tsx";
import ProfileLists from "../profiles/ProfileLists.tsx";

export default function MinecraftFinder({osType}:{osType:string}){
    // @ts-ignore
    const listProfiles= async () => {
        await invoke("read_sftp_dir", {path:"/profiles/"})
    }
    const pathInfo = useQuery("defaultPath",async ():Promise<{defaultPath:string,readPath:FileEntry[],minecraftExists:boolean}> => {
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
        let readPath = await readDir(defaultPath+".minecraft")
        let minecraftExists = await exists(defaultPath+".minecraft")
        return({defaultPath:defaultPath+".minecraft",readPath,minecraftExists})
    },{initialData:{defaultPath:"",readPath:[],minecraftExists:false}})

    return(
        <>
            {/*<button type={"button"} onClick={listProfiles}>*/}
            {/*    View remote profiles*/}
            {/*</button>*/}
            {pathInfo.isLoading?
                <>Loading...</>
                :
                pathInfo.isError?
                    <>Something went wrong!</>
                    :
                    pathInfo.data?
                    <>
                        {/*Minecraft file found at {pathInfo.data.defaultPath}*/}
                        {pathInfo.data.defaultPath&&
                            <>
                                <div className={'flex'}>
                                    <ProfileLists/>
                                    <ProfileDisplay/>
                                </div>
                                {/*<ProfileInfo path={pathInfo.data.defaultPath}/>*/}
                                {/*<RemoteInfo path={pathInfo.data.defaultPath}/>*/}
                                {/*<MinecraftVersions path={pathInfo.data.defaultPath}/>*/}
                            </>
                        }
                        {/*{pathInfo.data.minecraftExists&&<p>it exists!</p>}*/}
                        {/*<ModsFolder path={pathInfo.data.defaultPath}/>*/}
                    </>
                        :
                        <>No Data!?</>
            }
        </>
    )
}