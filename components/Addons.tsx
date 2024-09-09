import {useMutation, useQuery, useQueryClient, UseQueryResult} from "react-query";
import {ProfileAddon} from "../lib/types";
import {invoke} from "@tauri-apps/api";
import LoadingSpinner from "./LoadingSpinner";
import AddonRow from "./profiles/AddonRow";
import {useEffect, useState} from "react";
import FileInput from "./inputs/FileInput";
import NewAddonRow from "./ModInstaller/NewAddonRow.tsx";
import {FilePath} from "./profiles/profileMods/ProfileMods.tsx";

export default function Addons({addonType}:Readonly<{ addonType:string }>){
    const [newAddons, setNewAddons] = useState<ProfileAddon[]|undefined>()
    const queryClient = useQueryClient();
    let remoteResourcePacks:UseQueryResult<ProfileAddon[]> = useQuery(["remote-resource-packs"],async () => {
        return await invoke("read_remote_addons",{addonType:addonType})
    })
    useEffect(()=>{
        if(remoteResourcePacks.data){
            console.log(remoteResourcePacks.data)
        }
    },[remoteResourcePacks])
    let mutation = useMutation(
        {
            mutationFn:async (addon: ProfileAddon) => {
                // let cleanAddon = {
                //     addonType:addon.addonType,
                //     name:addon.name,
                //     fileName:addon.fileName,
                //     dependencies:addon,
                //     location:addon.location,
                //     versions:addon.versions,
                // }
                // console.log(cleanAddon);
                return await invoke("update_profile_addon", {addon:addon, addonType: "ResourcePack"})
            },
            onSuccess:async () => {
                return await queryClient.refetchQueries("read_remote_resource_packs")

            }
        }
    )
    const fileHandler=(files:string[])=>{
        let addons:ProfileAddon[] = [];
        for (let file of files){
            let filePath = new FilePath(file)
            let {name,fileName} = filePath.getFileInfo();
            addons.push({
                addonType:addonType,
                dependencies: [],
                fileName,
                location: filePath.path,
                name,
                versions: []
            })
        }
        setNewAddons(addons)
    }

    const updateAddon=(addon:ProfileAddon)=>{
        mutation.mutate(addon);
        // await invoke("update_remote_addon",{addon,addonType:"ResourcePack"})
    }
    const installNewAddons=async () => {
        // console.log(newAddons)
        await invoke("upload_profile_addons",{addons:newAddons})
    }
    return(
        <div className={'flex flex-col'}>
            <div className={'flex flex-wrap relative'}>
                <h3 className={'text-center font-bold text-2xl w-full '}>Mods</h3>
                <FileInput fileHandler={fileHandler}/>
            </div>
            {newAddons?.map(addon=>{
                return(
                    <NewAddonRow addon={addon} newAddons={newAddons}  updateAddon={updateAddon}/>
                )
            })}
            {newAddons && newAddons.length > 0&&
                <button onClick={installNewAddons}>
                    Install new mods
                </button>
            }
            {remoteResourcePacks.isLoading?
                    <LoadingSpinner/>
                :
                remoteResourcePacks.data?
                    <>
                        {remoteResourcePacks.data.map(resourcePack=>{
                            return(
                                <AddonRow addon={resourcePack} updateAddon={updateAddon}/>
                            )
                        })}
                    </>
                    :
                    remoteResourcePacks.error?
                        <>Error!</>
                        :
                        <>Something is very wrong</>
            }
        </div>
    )
}
