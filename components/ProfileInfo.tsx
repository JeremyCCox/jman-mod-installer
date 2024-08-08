import {useQuery, useQueryClient} from "react-query";
import {exists, readDir, createDir} from "@tauri-apps/api/fs";
import React, {FormEvent, useRef} from "react";
import LocalProfile from "../components/LocalProfile";
import {invoke} from "@tauri-apps/api";

export default function ProfileInfo({path}:Readonly<{path:string}>){
    const profileNameRef = useRef(null)
    const queryClient = useQueryClient()
    const newProfile=async (e: FormEvent<HTMLFormElement>)=>{
        e.preventDefault()
        let profileName = (e.currentTarget["profile_name"].value).replace(/[^a-z0-9.-]/gi, '_');
        if(profileInfo.data){
            let all_profiles = [...profileInfo.data?.remote_profiles, ...profileInfo.data?.local_profiles]
            console.log(all_profiles)
            if(!all_profiles.includes(profileName)){
                let modsFrom = e.currentTarget["mods_from"].value;
                if(modsFrom==="Start from scratch"){
                    await invoke("create_new_profile",{basePath:path , profileName}).then((res)=>{
                        console.log(res)
                        // setMessage(res)
                        queryClient.invalidateQueries("profiles")
                    }).catch(err=>{
                        // setMessage(err)
                    }).finally(()=>{
                        queryClient.refetchQueries("profiles")
                        // setLoading(false)
                    });
                }else{
                    await invoke("copy_profile",{basePath:path ,profileName,modsFrom}).then((res)=>{
                        console.log(res)
                        queryClient.invalidateQueries("profiles")
                    }).catch(err=>{
                        // setMessage(err)
                    }).finally(()=>{
                        queryClient.refetchQueries("profiles")
                        // setLoading(false)
                    });
                }
            }
        }


    }
    const profileInfo = useQuery('profiles',async () => {
            let profiles_exists = await exists(path);
            if(!profiles_exists){
                await createDir(`${path}/profiles`)
            }
            let remote_profiles = (await invoke<[string]>("read_profile_names"))
            let local_profiles  = (await readDir(`${path}/profiles`,{recursive:false})).map(local=>{
                return(local.name||crypto.randomUUID())
            })
            return({remote_profiles,local_profiles})
        }
        ,{refetchOnWindowFocus:false})

    // const localProfileInfo = useQuery('local_profiles',async () => {
    //     let profiles_exists = await exists(path);
    //     if(!profiles_exists){
    //         await createDir(`${path}/profiles`)
    //     }
    //     let profiles = (await readDir(path+"/profiles",{recursive:true})).map(profile=>{
    //         return betterReadout(profile)
    //     })
    //     return({profiles})
    // })
    // const remoteProfileInfo = useQuery('remote_profiles',async () => {
    //     let profiles_exists = await exists(path);
    //     if(!profiles_exists){
    //         await createDir(`${path}/profiles`)
    //     }
    //     let profiles = (await readDir(path+"/profiles",{recursive:true})).map(profile=>{
    //         return betterReadout(profile)
    //     })
    //     return({profiles})
    // })

    // const betterReadout = (listing : FileEntry)=>{
    //     let modsListing = listing.children?.find(({name})=>name === "mods")
    //     return({name:listing.name||"",mods:modsListing});
    // }
    return(
        <div className={'grid border-black border'}>
            <h2 className={'text-4xl text-center'}>Local Profiles</h2>
            <div className={'flex flex-wrap justify-evenly overflow-x-auto border'}>
                <form onSubmit={newProfile} className={"w-1/2 min-w-64 lg:min-w-1/4 lg:w-1/4 h-50 px-4 flex flex-col py-8 border-b-black border-b-4"} >
                    <h4 className={'text-xl font-bold text-center'}>New Profile</h4>
                        <input ref={profileNameRef} className={'w-full'} required type={'text'} name={"profile_name"} placeholder={"Profile Name"}/>
                        <button className={''}>
                            New Profile
                        </button>
                        <label className={'text-center grid w-full'}>
                            From:
                            <select name={'mods_from'}>
                                <option>
                                    Start from scratch
                                </option>
                                {profileInfo.data?.local_profiles?.map(profileName=>{
                                    return(
                                        <option key={profileName} value={profileName}>
                                            {profileName}
                                        </option>
                                    )
                                })}
                            </select>
                        </label>


                </form>
                {profileInfo.data&&
                    profileInfo.data.local_profiles.map(profile=>{
                        return(
                            <LocalProfile key={profile} profileName={profile} path={path}/>
                        )
                    })
                }
            </div>
        </div>
    )
}