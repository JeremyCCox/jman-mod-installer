import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";
import ModDiscrepancies from "./ModDiscrepancies.tsx";
import {Fragment, useState} from "react";
import LoadingSpinner from "./LoadingSpinner.tsx";
import {LocalProfile, ProfileAddon, RemoteProfile} from "../lib/types.ts";

export default function CompareLocalProfile({profileName}:Readonly<{ profileName:string}>){
    const queryClient = useQueryClient();
    const [loading, setLoading] = useState(false);
    const installMissingMods = async ()=>{
        setLoading(true)
        // @ts-ignore
        await invoke("install_specified_mods",{profileName,modsList:compareProfileInfo.data?.missing}).then((res)=>{
            // setMessage(res)
            // queryClient.refetchQueries(["local_profile", {name:profileName}])

            queryClient.invalidateQueries(["compare_profiles",profileName])
            queryClient.invalidateQueries(["remote_profiles",profileName])
        }).catch((err:string)=>{
            console.error(err)
            // setMessage(err)
        }).finally(()=>{
            queryClient.refetchQueries(["compare_profiles",profileName])
            queryClient.invalidateQueries(["remote_profiles",profileName])
            setLoading(false)
        })

    }
    const uploadAdditionalMods = async ()=>{
        setLoading(true)
        // @ts-ignore
        await invoke("upload_additional_mods",{profileName,modsList:compareProfileInfo.data?.extras}).then((res)=>{
            // setMessage(res)
            queryClient.invalidateQueries(["compare_profiles",profileName])
            queryClient.invalidateQueries(["remote_profiles",profileName])
        }).catch((err:string)=>{
            console.error("Error is: ",err)
            // setMessage(err)
        }).finally(()=>{
            queryClient.refetchQueries(["compare_profiles",profileName])
            queryClient.invalidateQueries(["remote_profiles",profileName])

            setLoading(false)
        })

    }
    const compareProfileInfo:UseQueryResult<{missing:ProfileAddon[],extras:ProfileAddon[]}> = useQuery(["compare_profiles",profileName],async () => {
        let remote = await invoke<RemoteProfile>("read_specific_remote_profile", {profileName})
        let local = await invoke<LocalProfile>("read_specific_local_profile", {profileName})
        let missing = remote.mods?.filter((remote) => !local.mods?.find(({name})=>name === remote.name));
        let extras = local.mods?.filter((local) => !remote.mods?.find(({name})=>name === local.name));
        if(!local.mods){
            missing = remote.mods;
        }
        if(!remote.mods){
            extras = local.mods;
        }
        console.log({missing,extras})
        return({missing,extras})
    })
    return(
        <Fragment key={profileName}>
            {compareProfileInfo.isLoading?
                <div className={'flex justify-center'}>
                    <LoadingSpinner/>
                </div>
                :
                compareProfileInfo.data !== undefined?
                    <>
                        {
                            compareProfileInfo.data.extras.length === 0 && compareProfileInfo.data.missing.length === 0 &&
                            <h3 className={'font-bold text-center text-green-500'}>This profile is up to date!</h3>
                        }
                        {compareProfileInfo.data.missing.length>0&&
                            <ModDiscrepancies modlist={compareProfileInfo.data.missing} notice={`You are missing ${compareProfileInfo.data.missing.length} mod${compareProfileInfo.data.missing.length>1?'s':''}!`} callback={installMissingMods} loading={loading} callbackTitle={"Install missing mods!"}/>
                        }
                        {compareProfileInfo.data.extras.length>0&&
                            <ModDiscrepancies modlist={compareProfileInfo.data.extras} notice={`You have ${compareProfileInfo.data.extras.length} new mod${compareProfileInfo.data.extras.length>1?'s':''} to upload!`} callback={uploadAdditionalMods} loading={loading}  callbackTitle={"Upload new mods!"}/>
                        }
                    </>
                    :
                    <h3>Something went wrong</h3>
            }

        </Fragment>
    )
}