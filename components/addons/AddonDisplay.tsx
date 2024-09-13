import {useMutation, useQuery, useQueryClient, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";
import {AddonType, ProfileAddon} from "../../lib/types.ts";
import {useSearchParams} from "react-router-dom";
import AddonDependencies from "./AddonDependencies.tsx";

export default function AddonDisplay({addonType}:Readonly<{ addonType:AddonType }>){
    const [searchParams,setSearchParams] = useSearchParams();
    const queryClient = useQueryClient();

    const addon:UseQueryResult<ProfileAddon> = useQuery(["profile_addon",searchParams.get("addon_name")],async () => {
        switch (searchParams.get("isNew")){
            case("t"):
                let data = queryClient.getQueryData(["new_addon",searchParams.get("addon_name")]);
                if(!data){
                    searchParams.delete("addon_name")
                    setSearchParams(searchParams);
                }
                return(data)
            case("f"):
                return(await invoke("read_remote_addon", {addonName: searchParams.get("addon_name"),addonType:addonType}))
            default:
                console.log('break')
        }
    })
    let newMutation = useMutation(
        {
            mutationFn:async(addon:ProfileAddon)=>{
                queryClient.setQueryData(["new_profile",searchParams.get("addon_name")],addon);
            },
            onSuccess:async () => {
                await queryClient.refetchQueries(["new_profile", searchParams.get("addon_name")])
            }
        }
    )

    let remoteMutation = useMutation(
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
                return await invoke("update_addon", {addon:addon})
            },
            onSuccess:async () => {
                return await queryClient.refetchQueries(["profile_addon",addon.data?.addonType])

            }
        }
    )
    const mutateAddon=(addon:ProfileAddon)=>{
        switch(searchParams.get("isNew")){
            case("t"):
                newMutation.mutate(addon);
                break
            case("f"):
               remoteMutation.mutate(addon);
                break
            default:
                console.log("default hit")
        }
    }


    return(
        <div className={'w-full min-h-[300px] bg-yellow-300 flex flex-col'}>
            <h2 className={'text-center font-bold'}>
                {searchParams.get("addon_name")}
            </h2>
            <AddonDependencies addon={addon} mutateAddon={mutateAddon}/>
        </div>
    )
}