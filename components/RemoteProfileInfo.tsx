import React, {useState} from "react";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import CompareLocalProfile from "./CompareLocalProfile.tsx";
import {invoke} from "@tauri-apps/api";
import LoadingSpinner from "./LoadingSpinner.tsx";
import {RemoteProfile} from "../lib/types.ts";

export default function RemoteProfileInfo({profileName}:Readonly<{profileName:string}>){
    const [ loading,setLoading] = useState(false)

    const queryClient = useQueryClient();

    const installProfile= async (e:React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
        setLoading(true)
        console.log(e.currentTarget.name);
        // @ts-ignore
        invoke('download_sftp_profile', {profileName: e.currentTarget.name}).then((res:string)=>{
            queryClient.invalidateQueries("profiles")
            queryClient.invalidateQueries(["list_local_profiles"])
            queryClient.invalidateQueries(["list_remote_profiles"])
        }).catch((err)=>{
            console.error(err)
        }).finally(()=>{

            queryClient.refetchQueries(["list_remote_profiles"])
            queryClient.refetchQueries(["list_local_profiles"])
            queryClient.refetchQueries("profiles")
            setLoading(false)
        })
    }
    const local_profiles:UseQueryResult<[string]> = useQuery(["list_local_profiles"],async () => {
        return await invoke<[string]>("list_local_profiles").then(res => {
            return res;
        }).catch(err => {
            console.error(err)
            return ([])
        })
    })
    const remoteProfile:UseQueryResult<RemoteProfile>=useQuery(["remote_profiles",profileName],async () => {
        return await invoke("read_specific_remote_profile", {profileName})
    },
        {enabled:!!profileName}
    )
    const openProfileLocation=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
        // @ts-ignore
        await invoke('profile_location',{profileName:e.currentTarget.name}).then((res)=>{
            console.log(res)
            // setMessage(res)
        }).catch(err=>{
            console.error(err)
        }).finally(()=>{
            setLoading(false)
        });
    }


    return(

            <div className={"w-1/2 min-w-64 lg:min-w-1/4 lg:w-1/4 h-50 px-4 flex flex-col py-8 border-b-black border-b-4"}>
                <h4 className={'text-xl font-bold text-center grid'}>
                    {profileName}{local_profiles.data&&local_profiles.data.includes(profileName)?<span className={'text-green-500 font-bold'}>Installed ✓</span>:<span className={'text-red-500'}>Not installed X</span>}
                </h4>
                {/*<button type={'button'} onClick={()=>{setInspectable(!inspectable)}}>*/}
                {/*    Verify Profile*/}
                {/*</button>*/}
                {/*<pre className={'text-wrap'}>*/}
                {/*    {JSON.stringify(remoteProfile,null,2)}*/}
                {/*</pre>*/}

                {(remoteProfile.isLoading||remoteProfile.data)&&
                        <div className={'h-60 w-full border-2 border-black overflow-y-auto'}>
                            {remoteProfile.isLoading&&
                                <div className={'text-center'}>
                                    <LoadingSpinner/>
                                </div>
                            }
                            {remoteProfile.data&&
                                <>
                                    <CompareLocalProfile profileName={remoteProfile.data.name} key={profileName} />
                                    <div className={'border border-black m-2'}>
                                        <h3 className={'text-center font-bold text-2xl'}>Mods</h3>
                                        {remoteProfile.data.mods?.map(mod=>{
                                            return(
                                                <p className={''} key={mod.name}>
                                                    {mod.name}
                                                </p>
                                            )
                                        })}
                                    </div>
                                </>

                            }
                        </div>
                }
                {local_profiles.data&&local_profiles.data.includes(profileName)?
                    <button className={'bg-green-400'} disabled={loading}  name={profileName} onClick={openProfileLocation} >
                        Open Local Location
                    </button>
                    :
                    <button className={'bg-blue-400'} name={profileName} onClick={installProfile} disabled={loading}>
                        {loading?<LoadingSpinner/>:"Download Profile"}
                    </button>
                }


            </div>


    )
}