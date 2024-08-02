import {useQuery, useQueryClient} from "react-query";
import {exists, readDir, createDir, FileEntry, removeDir} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";
import React, {useRef, useState} from "react";
import LoadingSpinner from "./LoadingSpinner.tsx";

export default function ProfileInfo({path}:Readonly<{path:string}>){
    const [loading,setLoading] =useState(false);
    const [message,setMessage] =useState("");
    const profileNameRef = useRef(null)
    const queryClient = useQueryClient();

    const listProfileMods= async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
        setLoading(true)
        // @ts-ignore
        await invoke('upload_sftp_dir', {basePath: path, profileName: e.currentTarget.name}).then((res:string)=>{
            setMessage(res)
        }).catch(err=>{
            setMessage(err)
        }).finally(()=>{
            setLoading(false)
        });
        setLoading(false)
    }
    const deleteProfile=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
       // @ts-ignore
        removeDir(`${path}\\profiles\\${e.currentTarget.name}`,{recursive:true}).then((res:string)=>{
            setMessage(res)
            queryClient.refetchQueries("profiles")
        }).catch(err=>{
            setMessage(err)
        }).finally(()=>{
            setLoading(false)
        })
    }
    const newProfile=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        e.preventDefault()
        setLoading(true)
        let element : HTMLElement | null = document.getElementById('new-profile');
        if(element){

            // @ts-ignore
            await invoke("create_new_profile",{basePath:path , profileName:element.value}).then((res:string)=>{
                console.log(res)
                // setMessage(res)
            }).catch(err=>{
                setMessage(err)
            }).finally(()=>{
                setLoading(false)
            });
        }
    }
    const copyProfile=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
        e.preventDefault()
        setTimeout(()=>{
            setLoading(false)

        },7000)
        // console.log(await copyFile(`${path}\\profiles\\${e.currentTarget.name}`,`${path}\\profiles\\${e.currentTarget.name}-copy`,{recursive:true}));
    }
    const openProfileLocation=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
        // @ts-ignore
        await invoke('profile_location',{basePath:path,profileName:e.currentTarget.name}).then((res)=>{
            console.log(res)
            // setMessage(res)
        }).catch(err=>{
            setMessage(err)
        }).finally(()=>{
            setLoading(false)
        });
    }
    const profileInfo = useQuery('local_profiles',async () => {
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
            <div className={'w-full'}>
                <div className={'grid justify-evenly'}>
                    <h2 className={'text-4xl text-center'}>Local Profiles</h2>
                    <div>
                        <input ref={profileNameRef} type={'text'} id={"new-profile"} placeholder={"Profile Name"}/>
                        <button className={''} onClick={newProfile}>
                            New Profile
                        </button>
                    </div>

                </div>
                    <span className={'flex min-h-[80px] justify-center'}>
                        {loading?
                            <LoadingSpinner/>
                            :
                            <p className={'padding 4px'}>
                                {message}
                            </p>
                        }
                    </span>
            </div>
            <div className={'flex justify-evenly overflow-x-auto'}>
                {profileInfo.data&&
                    profileInfo.data.profiles.map(profile=>{
                        return(
                            <div key={profile.name} className={"w-full h-50 px-4 grid justify-evenly"}>
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
                                    <button className={'w-full rounded-sm m-2'} disabled={loading} name={profile.name} type={'button'} onClick={listProfileMods}>
                                        Upload profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} disabled={loading} name={profile.name} type={'button'} onClick={deleteProfile}>
                                        Delete profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} disabled={loading} name={profile.name} type={'button'} onClick={copyProfile} >
                                        Copy Profile
                                    </button>
                                    <button className={'w-full rounded-sm m-2'} disabled={loading}  name={profile.name} onClick={openProfileLocation} >
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