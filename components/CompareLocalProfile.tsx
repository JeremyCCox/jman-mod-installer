import {RemoteProfile} from "@my-types/";
import {useQuery, useQueryClient} from "react-query";
import {FileEntry, readDir} from "@tauri-apps/api/fs";
import {invoke} from "@tauri-apps/api";
import ModDiscrepancies from "./ModDiscrepancies.tsx";

export default function CompareLocalProfile({profile,path}:Readonly<{ profile:RemoteProfile,path:string }>){
const queryClient = useQueryClient();
    const installMissingMods = async ()=>{
        // @ts-ignore
        await invoke("install_missing_mods",{basePath:path,profileName:profile.name,missingMods:localProfileInfo.data?.missing}).then((res)=>{
            // setMessage(res)
            queryClient.refetchQueries(["local_profile", {name:profile.name}])
        }).catch((err:string)=>{
            console.error(err)
            // setMessage(err)
        }).finally(()=>{
            // setLoading(false)
        })

    }
    const uploadAdditionalMods = async ()=>{
        // @ts-ignore
        await invoke("upload_additional_mods",{basePath:path,profileName:profile.name,missingMods:localProfileInfo.data?.extras}).then((res)=>{
            // setMessage(res)
            queryClient.refetchQueries(["local_profile", {name:profile.name}])
        }).catch((err:string)=>{
            console.error(err)
            // setMessage(err)
        }).finally(()=>{
            // setLoading(false)
        })

    }

    const localProfileInfo = useQuery(["local_profile", {name:profile.name}],async () => {

        let returnVal= await readDir(`${path}/profiles/${profile.name}/mods`, {recursive: true}).then(res=>{
            let mods:string[] = res.map(profile=>{
                return betterReadout(profile)
            }) || [""]
            let missing = profile.mods?.filter(x=>!mods.includes(x));
            if (!profile.mods){
                return ({mods,missing})
            }
            let extras = mods?.filter(x=>!profile.mods?.includes(x));
            return ({mods,missing,extras})
        }).catch(err=>{
            console.error(err)
        }).finally(()=>{

        })
        console.log(returnVal)
        return returnVal
    })
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
            {localProfileInfo.data?.missing&&localProfileInfo.data.missing.length>0&&
                <ModDiscrepancies modlist={localProfileInfo.data.missing} notice={`You are missing ${localProfileInfo.data.missing.length} mod${localProfileInfo.data.missing.length>1?'s':''}!`} callback={installMissingMods} callbackTitle={"Install missing mods!"}/>
            }

            {localProfileInfo.data?.extras&&localProfileInfo.data.extras.length>0&&
                <ModDiscrepancies modlist={localProfileInfo.data.extras} notice={`You have ${localProfileInfo.data.extras.length} new mod${localProfileInfo.data.extras.length>1?'s':''} to upload!`} callback={uploadAdditionalMods} callbackTitle={"Upload new mods!"}/>
            }
        </>
    )
}