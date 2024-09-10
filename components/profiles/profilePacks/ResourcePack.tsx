import {invoke} from "@tauri-apps/api";
import {useSearchParams} from "react-router-dom";
import {useQueryClient} from "react-query";
import {ProfileAddon} from "../../../lib/types.ts";

export default function ResourcePack({pack,installed}:Readonly<{pack:ProfileAddon,installed:boolean}>){
    let [searchParams] = useSearchParams();
    const queryClient = useQueryClient();
    const installPack=async ()=>{
        await invoke("install_resource_pack", {profileName:searchParams.get("profile"),packName:pack.name})
        await queryClient.refetchQueries(["local-profiles",searchParams.get("profile")])
    }
    const deletePack=async ()=>{
        await invoke("remove_local_resource_pack", {profileName:searchParams.get("profile"),packName:pack.name});
        await queryClient.refetchQueries(["local-profiles",searchParams.get("profile")])
    }
    if(installed){
        return(
            <div className={'text-green-500 flex items-center relative'}>
                <p>
                    {pack.name}
                </p>
                <span className={'w-8 text-center font-bold text-lg'}>
                    {pack.dependencies?.length}
                </span>
                <button className={'max-w-[15%]'} onClick={deletePack}>Delete</button>
                <div className={'top-full w-full  border-black border-2 bg-gray-300'}>
                    {pack.dependencies?.map(value => {
                        return (<span>{value}</span>)
                    })}
                </div>
            </div>
        )
    }else{
        return(
            <div className={'text-red-500 flex '}>
                <p>
                    {pack.name}
                </p>
                <button onClick={installPack}>Install</button>
            </div>
        )
    }
}