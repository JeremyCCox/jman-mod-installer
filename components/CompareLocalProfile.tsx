import {RemoteProfile} from "@my-types/";
import {useQuery, useQueryClient, UseQueryResult} from "react-query";
import {FileEntry, readDir} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";
import ModDiscrepancies from "./ModDiscrepancies.tsx";
import {useState} from "react";

export default function CompareLocalProfile({profileName,path}:Readonly<{ profileName:string,path?:string }>){
    const queryClient = useQueryClient();
    const [loading, setLoading] = useState(false);
    const installMissingMods = async ()=>{
        setLoading(true)
        // @ts-ignore
        await invoke("install_missing_mods",{basePath:path,profileName,missingMods:localProfileInfo.data?.missing}).then((res)=>{
            // setMessage(res)
            queryClient.refetchQueries(["local_profile", {name:profileName}])
        }).catch((err:string)=>{
            console.error(err)
            // setMessage(err)
        }).finally(()=>{
            setLoading(false)
        })

    }
    const uploadAdditionalMods = async ()=>{
        setLoading(true)
        // @ts-ignore
        await invoke("upload_additional_mods",{basePath:path,profileName:profile.name,missingMods:localProfileInfo.data?.extras}).then((res)=>{
            // setMessage(res)
            queryClient.refetchQueries(["local_profile", {name:profileName}])
        }).catch((err:string)=>{
            console.error(err)
            // setMessage(err)
        }).finally(()=>{
            setLoading(false)
        })

    }
    const remoteProfile:UseQueryResult<RemoteProfile>=useQuery(["remote_profiles",profileName],async () => {
            return await invoke("read_specific_remote_profile", {profileName})
        },
        {enabled:!!profileName}
    )

    const localProfileInfo = useQuery(["local_profile", {name:profileName}],async () => {

        let returnVal= await readDir(`${path}/profiles/${profileName}/mods`, {recursive: true}).then(res=>{
            let mods:string[] = res.map(profile=>{
                return betterReadout(profile)
            }) || [""]
            let missing = remoteProfile.data?.mods?.filter(x=>!mods.includes(x));
            let extras = mods?.filter(x=>!remoteProfile.data?.mods?.includes(x));
            return ({mods,missing,extras})
        }).catch(err=>{
            console.error(err)
        }).finally(()=>{

        })
        return returnVal
    },
        {enabled:!!remoteProfile.data}
    )
    // useEffect(()=>{
    //     if(!localProfileInfo.isLoading){}
    //     console.log(localProfileInfo.data?.profiles[0]);
    //
    // },[localProfileInfo])

    const betterReadout=(listing:FileEntry)=> {

        return(listing.name||"");
    }

    return(
        <>
            {localProfileInfo.data?.missing&&localProfileInfo.data.missing.length==0&&localProfileInfo.data.extras.length==0&&
                <h3 className={'font-bold text-center text-green-500'}>This profile is up to date!</h3>
            }
            {localProfileInfo.data?.missing&&localProfileInfo.data.missing.length>0&&
                <ModDiscrepancies modlist={localProfileInfo.data.missing} notice={`You are missing ${localProfileInfo.data.missing.length} mod${localProfileInfo.data.missing.length>1?'s':''}!`} callback={installMissingMods} loading={loading} callbackTitle={"Install missing mods!"}/>
            }
            {localProfileInfo.data?.extras&&localProfileInfo.data.extras.length>0&&
                <ModDiscrepancies modlist={localProfileInfo.data.extras} notice={`You have ${localProfileInfo.data.extras.length} new mod${localProfileInfo.data.extras.length>1?'s':''} to upload!`} callback={uploadAdditionalMods} loading={loading}  callbackTitle={"Upload new mods!"}/>
            }
        </>
    )
}