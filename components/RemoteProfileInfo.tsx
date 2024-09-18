import React, {useState} from "react";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import CompareLocalProfile from "./CompareLocalProfile.tsx";
import {invoke} from "@tauri-apps/api";
import LoadingSpinner from "./LoadingSpinner.tsx";
import { ProfileAddon, RemoteProfile} from "../lib/types.ts";
import ProfileAddonRow from "./profiles/ProfileAddon.tsx";

export default function RemoteProfileInfo({profileName}:Readonly<{profileName:string}>){
    const [ loading,setLoading] = useState(false)

    const queryClient = useQueryClient();

    const installProfile= async (e:React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
        setLoading(true)
        // @ts-ignore
        await invoke('download_sftp_profile', {profileName: e.currentTarget.name}).catch(err=>{
            console.error(err)
        })
        await queryClient.refetchQueries(["list_remote_profiles"])
        await queryClient.refetchQueries(["list_local_profiles"])
        await queryClient.refetchQueries("profiles")
        setLoading(false)
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
    const removeAddon=async (addon:ProfileAddon)=>{
        await invoke("remove_addon_from_remote_profile", {profileName, addon,addonType:addon.addonType})
        await queryClient.refetchQueries(["remote_profiles",profileName])
        await queryClient.refetchQueries(["list_local_profiles",profileName])
    }

    return(

            <div className={" h-50 px-4 flex flex-col py-8 border-b-black border-b-4"}>
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
                        <div className={'h-[60vh] w-full border-2 border-black overflow-y-auto'}>
                            {remoteProfile.isLoading&&
                                <div className={'text-center'}>
                                    <LoadingSpinner/>
                                </div>
                            }
                            {remoteProfile.data&&
                                <>
                                {local_profiles.data?.includes(profileName)&&
                                    <CompareLocalProfile profileName={remoteProfile.data.name} key={profileName} />
                                }
                                    <div className={'border border-black m-2'}>
                                        <h3 className={'text-center font-bold text-2xl'}>Mods</h3>
                                        {remoteProfile.data.mods?.map(addon=>{
                                            return(
                                                <ProfileAddonRow addon={addon} deleteFn={removeAddon}/>
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