import React, {useState} from "react";
import {invoke} from "@tauri-apps/api";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import CompareLocalProfile from "./CompareLocalProfile.tsx";
import {RemoteProfile} from "@my-types/*";
import LoadingSpinner from "./LoadingSpinner.tsx";
import {useSearchParams} from "react-router-dom";

export default function LocalProfile({profileName}:Readonly<{ profileName: string}>){
    const queryClient = useQueryClient();
    const [searchParams,setSearchParams] = useSearchParams()
    const [loading,setLoading] =useState(false);
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
        }).finally(()=>{
            setLoading(false)
        });
    }
    const listProfileMods= async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
        setLoading(true)
        // @ts-ignore
        await invoke('upload_local_profile', {profileName: e.currentTarget.name}).then((res:string)=>{
            queryClient.invalidateQueries("profiles")
        }).catch(err=>{

        }).finally(()=>{
            queryClient.refetchQueries("profiles")
            setLoading(false)
        });
        setLoading(false)
    }
    const deleteProfile=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
        // @ts-ignore
        invoke(`delete_local_profile`,{profileName:e.currentTarget.name}).then((res:string)=>{
            queryClient.invalidateQueries("profiles")
        }).catch(err=>{
        }).finally(()=>{
            setTimeout(()=>{
                queryClient.refetchQueries("profiles")
                searchParams.set("profile","")
                setSearchParams(searchParams)
            },1500)
            setLoading(false)
        })
    }
    const remoteProfile:UseQueryResult<RemoteProfile>=useQuery(["remote_profiles",profileName],async () => {
            return await invoke("read_specific_remote_profile", {profileName})
        },
        {enabled:!!profileName}
    )


    return(
        <>
            <h3 className={'text-2xl font-bold text-center grid w-full'}>
                {profileName}
            </h3>
            <div className={'flex flex-wrap justify-evenly w-full '}>
                <button className={'w-24 h-24 m-2'} disabled={loading} name={profileName} type={'button'} onClick={listProfileMods}>
                    Upload
                </button>
                <button className={'w-24 h-24 m-2'} disabled={loading} name={profileName} type={'button'} onClick={deleteProfile}>
                    Delete
                </button>
                <button className={'w-24 h-24 m-2'} disabled={loading} name={profileName} type={'button'} onClick={copyProfile} >
                    Copy
                </button>
                <button className={'w-24 h-24 m-2'} disabled={loading}  name={profileName} onClick={openProfileLocation} >
                    Open
                </button>
            </div>
                <CompareLocalProfile profileName={profileName}/>
                <div className={'border w-full border-black m-2'}>
                    {remoteProfile.data&&
                        <div className={'h-60 w-full border-2 border-black overflow-y-auto'}>
                            {remoteProfile.isLoading&&
                                <div className={'text-center'}>
                                    <LoadingSpinner/>
                                </div>
                            }
                            {remoteProfile.data&&

                                    <div className={'m-2'}>
                                        <h3 className={'text-center font-bold text-2xl'}>Mods</h3>
                                        {remoteProfile.data.mods?.map(mod=>{
                                            return(
                                                <p className={''} key={mod}>
                                                    {mod}
                                                </p>
                                            )
                                        })}
                                    </div>
                            }
                        </div>
                    }
                </div>

        </>
    )
}