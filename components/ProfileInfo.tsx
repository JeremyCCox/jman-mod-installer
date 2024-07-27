import {useQuery} from "react-query";
import {exists, readDir, createDir, FileEntry, removeDir, copyFile} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";

export default function ProfileInfo({path}:Readonly<{path:string}>){
    const listProfileMods= async (name: string | undefined) => {
        console.log(await invoke('upload_sftp_dir', {basePath: path, profileName: name}))
    }
    const deleteProfile=async (name:string)=>{
        console.log(await removeDir(`${path}\\profiles\\${name}`,{recursive:true}));
    }
    const copyProfile=async (name:string)=>{
        console.log(await copyFile(`${path}\\profiles\\${name}`,`${path}\\profiles\\${name}-copy`,{recursive:true}));
    }
    const openProfileLocation=async (name:string)=>{
        console.log(await invoke('profile_location',{basePath:path,profileName:name}));
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
            <h2>Local Profiles</h2>
            <div className={'flex justify-evenly overflow-x-auto'}>
                {profileInfo.data&&
                    profileInfo.data.profiles.map(profile=>{
                        return(
                            <div key={profile.name} className={'border border-black px-4'}>
                                <h4 className={'text-xl font-bold text-center'}>{profile.name}</h4>
                                <div className={'max-h-60 border-2 border-black overflow-y-scroll'}>
                                    {profile.mods?.children?.map(mod=>{
                                        return(
                                            <p className={''} key={mod.name}>
                                                {mod.name}
                                            </p>
                                        )
                                    })}
                                </div>
                                <div className={'flex flex-wrap justify-evenly w-full '}>
                                    <button className={'w-full rounded-sm m-2'} type={'button'} onClick={()=>listProfileMods(profile.name)}>
                                        Upload profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} type={'button'} onClick={()=>deleteProfile(profile.name)}>
                                        Delete profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} type={'button'} onClick={()=>copyProfile(profile.name)} >
                                        Copy Profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} onClick={()=>openProfileLocation(profile.name)} >
                                        Open Location
                                    </button>
                                </div>
                            </div>
                        )
                    })
                }
            </div>
        </div>
    )
}