import React, {useState} from "react";
import {invoke} from "@tauri-apps/api";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import CompareLocalProfile from "./CompareLocalProfile.tsx";
import LoadingSpinner from "./LoadingSpinner.tsx";
import {useSearchParams} from "react-router-dom";
import ProfileMods from "./profiles/profileMods/ProfileMods.tsx";
import ProfileResourcePacks from "./profiles/profilePacks/ProfileResourcePacks.tsx";
import {RemoteProfile} from "../lib/types.ts";

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
        await invoke('profile_location',{profileName:searchParams.get("profile")}).then((res)=>{
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
    const localProfile:UseQueryResult<RemoteProfile>=useQuery(["local-profiles",profileName],async () => {
            return await invoke("read_specific_local_profile", {profileName})
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
                    {localProfile.data&&
                        <div className={''}>
                            {localProfile.isLoading&&
                                <div className={'text-center'}>
                                    <LoadingSpinner/>
                                </div>
                            }
                            <ProfileMods mods={localProfile.data?.mods}/>
                            <ProfileResourcePacks resourcePacks={localProfile.data?.resourcePacks}/>
                        </div>
                    }
                </div>

        </>
    )
}