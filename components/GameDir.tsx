import {useQuery, useQueryClient} from "react-query";
import {type} from "@tauri-apps/api/os";
import {exists, FileEntry, readDir} from "@tauri-apps/api/fs";
import {dataDir, homeDir} from "@tauri-apps/api/path";
import {open} from "@tauri-apps/api/dialog";

export default function GameDir(){
    const queryClient = useQueryClient();
    const osInfo = useQuery('osType',async () => {
        return (await type())
    })
    const pathInfo = useQuery("defaultPath",async ():Promise<{dataPath:string,defaultPath:string,readPath:FileEntry[],minecraftExists:boolean}> => {
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
        return({dataPath:defaultPath,defaultPath:defaultPath+".minecraft",readPath,minecraftExists})
    },{initialData:{dataPath:"",defaultPath:"",readPath:[],minecraftExists:false},enabled:!!osInfo.data});
    const openDialog = async ()=>{
        const val = await open({title:"Locate the .minecraft folder",multiple:false,directory:true,defaultPath:pathInfo.data?.dataPath})
        if(typeof val === "string"){
            let readPath = await readDir(val)
            let minecraftExists = await exists(val)
            let va2 = {dataPath:pathInfo.data?.dataPath,defaultPath:val,readPath,minecraftExists};
            console.log(va2)
            queryClient.setQueryData("defaultPath",va2)
        }
    }
    return(
        <>
            {pathInfo.data&&
                <>
                    <input type={'text'} hidden={true} value={pathInfo.data.defaultPath} name={"defaultGameDir"}/>
                    {
                        pathInfo.data.defaultPath.slice(pathInfo.data.defaultPath?.length-10)===".minecraft"?
                            <div>
                                <p>{pathInfo.data.defaultPath}</p>
                                <p className={'mb-0 pb-0'}>Does this directory not look correct?</p>
                            </div>
                            :
                            <p>
                                <p className={'text-red-500'}>{pathInfo.data.defaultPath}</p>
                                <p className={'mb-0 pb-0'}>This directory does not look correct!</p>
                            </p>
                    }
                </>
            }
            <button className={'p-0 mb-2 w-fit mx-auto'} type={"button"} onClick={openDialog}>Locate the .minecraft folder manually.</button>

        </>
    )
}