import {useQuery} from "react-query";
import {exists, readDir, createDir, FileEntry} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";

export default function ProfileInfo({path}:Readonly<{path:string}>){
    const listProfileMods= async (name: string | undefined) => {
        console.log(await invoke('upload_sftp_dir', {basePath: path, profileName: name}))
    }
    const profileInfo = useQuery('profiles',async () => {
        let profiles_exists = await exists(path);
        if(!profiles_exists){
            await createDir(`${path}/profiles`)
        }
        let profiles = (await readDir(path+"/profiles",{recursive:true})).map(profile=>{
            return betterReadout(profile)
        })
        return({profiles})
    })
    const betterReadout = (listing : FileEntry)=>{
        let modsListing = listing.children?.find(({name})=>name === "mods")
        return({name:listing.name,mods:modsListing});
    }
    return(
        <div className={'grid border-black border'}>
            {profileInfo.data&&
                profileInfo.data.profiles.map(profile=>{
                    return(
                        <div key={profile.name} className={'grid justify-evenly'}>
                            <h4 className={'text-xl font-bold text-center'}>{profile.name}</h4>
                            <div className={'max-h-60 border-2 border-black bg-amber-400 overflow-y-scroll'}>
                                {profile.mods?.children?.map(mod=>{
                                    return(
                                        <p className={''} key={mod.name}>
                                            {mod.name}
                                        </p>
                                    )
                                })}
                            </div>
                            <button type={'button'} onClick={()=>listProfileMods(profile.name)}>
                                list dir
                            </button>
                        </div>
                    )
                })
            }
        </div>
    )
}