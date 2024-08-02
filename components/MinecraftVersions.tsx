import {useQuery} from "react-query";
import {createDir, exists, FileEntry, readDir} from "@tauri-apps/api/fs";

export default function MinecraftVersions({path}:Readonly<{path:string}>){

    const versionInfo = useQuery('versions',async () => {
        let versions_exists = await exists(path);
        if(!versions_exists){
            await createDir(`${path}/profiles`)
        }
        let versions = (await readDir(path+"/versions",{recursive:true})).map(version=>{
            return betterReadout(version)
        })
        return({versions})
    })
    const betterReadout = (listing : FileEntry)=>{

        return({name:listing.name,mods:listing.path});
    }
    return(
        <div>
            <h2 className={'text-4xl text-center'}>Installed Minecraft Versions</h2>
            {
                versionInfo.data?.versions&&
                <>
                    <div className={'grid overflow-y-scroll max-h-60'}>
                        {versionInfo.data.versions.map(version=>{
                            return(
                                <div className={'w-full p-1 m-1 border flex justify-evenly'}>
                                    <h2 className={'text-2xl my-auto'}>{version.name}</h2>
                                    <button>
                                        Delete Version
                                    </button>
                                </div>
                            )
                        })}
                    </div>
                    {/*<pre>*/}
                    {/*    {JSON.stringify(versionInfo.data?.versions,null,2)}*/}
                    {/*</pre>*/}
                </>
            }
        </div>
    )
}