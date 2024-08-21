import {QueryClientProvider, useQuery, useQueryClient} from "react-query";
import {exists, createDir, readDir} from "@tauri-apps/api/fs";
import React, {Fragment, useEffect, useState} from "react";
import LoadingSpinner from "./LoadingSpinner.tsx";
import RemoteProfileInfo from "./RemoteProfileInfo.tsx";
import {invoke} from "@tauri-apps/api";


export default function RemoteInfo({path}:Readonly<{path:string}>){
    const [message, setMessage] = useState("")
    const [loading, setLoading] = useState(false);

    // const fakeProfiles=async ():Promise<[RemoteProfileType]>=>{
    //     return new Promise((resolve)=>{
    //         resolve(
    //             JSON.parse("[{\"name\":\"jman_modpack\",\"mods\":[\"sodium-fabric-0.5.11+mc1.20.1.jar\",\"twigs-3.1.0-fabric.jar\",\"mysticaloaktree-1.20-1.11-fabric.jar\",\"cloth-config-11.1.118-fabric.jar\",\"TerraBlender-fabric-1.20.1-3.0.1.7.jar\",\"better-end-4.0.11.jar\",\"lithium-fabric-mc1.20.1-0.11.2.jar\",\"naturalist-fabric-4.0.3-1.20.1.jar\",\"Ribbits-1.20.1-Fabric-3.0.0.jar\",\"geckolib-fabric-1.20.1-4.4.7.jar\",\"voicechat-fabric-1.20.1-2.5.19.jar\",\"RoguelikeDungeons-2.0.4-build4-1.20.1-fabric.jar\",\"jei-1.20.1-fabric-15.8.2.23.jar\",\"YungsApi-1.20-Fabric-4.0.5.jar\",\"fabric-seasons-2.3+1.20.jar\",\"Towns-and-Towers-1.12-Fabric+Forge.jar\",\"waystones-fabric-1.20-14.1.4.jar\",\"balm-fabric-1.20.1-7.3.6.jar\",\"InventorySorter-1.9.0-1.20.jar\",\"fabric-api-0.92.2+1.20.1.jar\",\"mcw-furniture-3.2.2-mc1.20.1fabric.jar\",\"Terralith_1.20.x_v2.5.4.jar\",\"ForgeConfigAPIPort-v8.0.0-1.20.1-Fabric.jar\",\"YungsBetterDungeons-1.20-Fabric-4.0.4.jar\",\"moonlight-1.20-2.12.9-fabric.jar\",\"bclib-3.0.14.jar\",\"CopperandTuffBackport-1.2.jar\",\"Excessive Building [Fabric] 1.20.1-2.0.4.jar\",\"decorative_blocks-fabric-1.20.1-4.1.3.jar\",\"cristellib-1.1.5-fabric.jar\",\"abverticaledition-1.0.3b-fabric-mc1.20.jar\",\"RegionsUnexploredFabric-0.5.6+1.20.1.jar\"],\"launcher_profile\":{\"created\":\"2024-07-28T20:12:59Z\",\"lastVersionId\":\"fabric-loader-0.15.11-1.20.1\",\"name\":\"jman_modpack\"}},{\"name\":\"new_profile\",\"mods\":[\"testjar.jar\"],\"launcher_profile\":{\"created\":\"2024-07-29T19:49:02Z\",\"lastVersionId\":\"fabric-loader-0.15.11-1.20.1\",\"name\":\"new_profile\"}},{\"name\":\"fabric-1.20.1\",\"mods\":[\"sodium-fabric-0.5.11+mc1.20.1.jar\",\"twigs-3.1.0-fabric.jar\",\"cloth-config-11.1.118-fabric.jar\",\"TerraBlender-fabric-1.20.1-3.0.1.7.jar\",\"lithium-fabric-mc1.20.1-0.11.2.jar\",\"naturalist-fabric-4.0.3-1.20.1.jar\",\"Ribbits-1.20.1-Fabric-3.0.0.jar\",\"geckolib-fabric-1.20.1-4.4.7.jar\",\"RoguelikeDungeons-2.0.4-build4-1.20.1-fabric.jar\",\"YungsApi-1.20-Fabric-4.0.5.jar\",\"InventorySorter-1.9.0-1.20.jar\",\"fabric-api-0.92.2+1.20.1.jar\",\"mcw-furniture-3.2.2-mc1.20.1fabric.jar\",\"ForgeConfigAPIPort-v8.0.0-1.20.1-Fabric.jar\",\"YungsBetterDungeons-1.20-Fabric-4.0.4.jar\",\"CopperandTuffBackport-1.2.jar\",\"Excessive Building [Fabric] 1.20.1-2.0.4.jar\",\"decorative_blocks-fabric-1.20.1-4.1.3.jar\",\"abverticaledition-1.0.3b-fabric-mc1.20.jar\",\"RegionsUnexploredFabric-0.5.6+1.20.1.jar\"],\"launcher_profile\":{\"created\":\"2024-07-28T02:07:34Z\",\"lastVersionId\":\"fabric-loader-0.15.11-1.20.1\",\"name\":\"fabric-1.20.1\"}}]")
    //         )
    //     })
    //
    // }


    /////////////

    // const profileInfo = useQuery('remote-profiles',async () => {
    //     let profiles_exists = await exists(path);
    //     if(!profiles_exists){
    //         await createDir(`${path}/profiles`)
    //     }
    //     let profiles = (await invoke<[RemoteProfile]>("read_sftp_dir",{}))
    //
    //     return({profiles})
    // }

    /////////////

    const profileInfo = useQuery('profiles',async () => {
            let profiles_exists = await exists(path);
            if(!profiles_exists){
                await createDir(`${path}/profiles`)
            }
            let remote_profiles = (await invoke<[string]>("list_remote_profiles"))
            let local_profiles  = (await readDir(`${path}/profiles`,{recursive:false})).map(local=>{
                return(local.name)
            })
            console.log("local profiles",local_profiles)
            return({remote_profiles,local_profiles})
        }
    ,{refetchOnWindowFocus:false})

    // useEffect(()=>{
    //     console.log("profile info",profileInfo)
    // },[profileInfo])
    // const betterReadout = (profile )=>{
    //     // let modsListing = listing.children?.find(({name})=>name === "mods")
    //     return("ada");
    // }
    return(
        <div className={'grid border-black border'}>
            <div className={'w-full'}>
                <div className={'grid justify-evenly'}>
                    <h2 className={'text-4xl text-center'}>Remote Profiles</h2>
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
            <div className={'flex flex-wrap justify-evenly overflow-x-auto border'}>
                {profileInfo.isLoading&&
                    <LoadingSpinner/>
                }
                {profileInfo.data&&
                    profileInfo.data.remote_profiles.map(profile=>{
                        return(
                            <Fragment key={`${profile}-${profileInfo.data.local_profiles.includes(profile)}`}>
                                <RemoteProfileInfo profileName={profile} installed={profileInfo.data.local_profiles.includes(profile)} path={path} setMessage={setMessage} />
                            </Fragment>
                            )
                    })
                }
            </div>
        </div>
    )
}