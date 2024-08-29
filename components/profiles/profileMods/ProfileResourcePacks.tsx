import React, {useEffect} from "react";
import {useQuery} from "react-query";
import {invoke} from "@tauri-apps/api";

export default function ProfileResourcePacks({resourcePacks}:Readonly<{resourcePacks:any}>){
    console.log(resourcePacks)
    let remoteResourcePacks = useQuery(["remote-resource-packs"],async () => {
        return await invoke("read_remote_resource_packs")
    })

    useEffect(()=>{
        console.log(remoteResourcePacks)
    },[remoteResourcePacks])
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
                            <p className={''} style={{color:resourcePacks.find(({name})=>name === resourcePack.name) === -1?"red":"green"}} key={resourcePack.name}>
                                {resourcePack.name}
                            </p>
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