import {useQuery, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";
import ResourcePack from "./ResourcePack.tsx";
import {ProfileAddon} from "../../../lib/types.ts";

export default function ProfileResourcePacks({resourcePacks}:Readonly<{resourcePacks?:ProfileAddon[]}>){
    let remoteResourcePacks:UseQueryResult<ProfileAddon[]> = useQuery(["remote-resource-packs"],async () => {
        return await invoke("read_remote_resource_packs")
    })

    if(resourcePacks){
        return(
            <div className={'m-2'}>
                <h3 className={'text-center font-bold text-2xl'}>Resource Packs</h3>
                <div className={'max-h-60 border-2 border-black overflow-y-auto'}>
                    {resourcePacks.map(resourcePack=>{
                        return(
                            <p className={''} key={resourcePack.name}>
                                {resourcePack.name}
                            </p>
                        )
                    })}
                </div>
                <div >
                    <h4 className={'text-center font-bold'}>Add Resource Pack</h4>
                    {remoteResourcePacks.data?.map(resourcePack=>{
                        return(
                            <ResourcePack pack={resourcePack} installed={!!resourcePacks.find(({name})=>name === resourcePack.name)}/>
                        )

                    })}
                </div>
            </div>
        )
    }else{

        return(
            <>
                None!
            </>
        )
    }
}