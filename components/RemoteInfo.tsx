import {useQuery} from "react-query";
import {exists, createDir} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";

export default function RemoteInfo({path}:Readonly<{path:string}>){
    const listProfileMods= async (name: string | undefined) => {
        console.log(await invoke('download_sftp_profile', {basePath: path, profileName: name}))
    }
    const profileInfo = useQuery('remote-profiles',async () => {
        let profiles_exists = await exists(path);
        if(!profiles_exists){
            await createDir(`${path}/profiles`)
        }
        let profiles = (await invoke<[string]>("read_sftp_dir",{path:"/upload/profiles"})).map(profile=>{
            return betterReadout(profile)
        })
        return({profiles})
    })
    const betterReadout = (profile : string)=>{
        // let modsListing = listing.children?.find(({name})=>name === "mods")
        return(profile.split("\\").pop());
    }
    return(
        <div className={'grid border-black border'}>
            <h2>Remote Profiles</h2>
            <div className={'flex overflow-x-auto py-2'}>
                {profileInfo.data&&
                    profileInfo.data.profiles.map(profile=>{
                        return(
                            <div key={profile} className={'w-1/3 h-50 px-4 grid justify-evenly'}>
                                <h4 className={'text-xl h-[3lh] font-bold text-center'}>{profile}</h4>
                                <div className={'h-fit border-2 border-black bg-amber-400'}>
                                    <button className={'rounded-none '} type={'button'} onClick={()=>listProfileMods(profile)}>
                                        Download Profile
                                    </button>
                                    {/*{profile.mods?.children?.map(mod=>{*/}
                                    {/*    return(*/}
                                    {/*        <p className={''} key={mod.name}>*/}
                                    {/*            {mod.name}*/}
                                    {/*        </p>*/}
                                    {/*    )*/}
                                    {/*})}*/}
                                </div>
                            </div>
                        )
                    })
                }
            </div>
        </div>
    )
}