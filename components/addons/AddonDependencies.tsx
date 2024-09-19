import { useQuery, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";
import {ProfileAddon} from "../../lib/types.ts";
import {useState} from "react";
import DependencyList from "./dependencies/DependencyList.tsx";

export default function AddonDependencies({addon,mutateAddon}:Readonly<{addon:UseQueryResult<ProfileAddon>,mutateAddon:(addon:ProfileAddon)=>void}>){
    const [toggleDep,setToggleDep] = useState(false)

    const allModsQuery=useQuery(["allMods"],async () => {
        return await invoke<ProfileAddon[]>("read_remote_addons",{addonType:"Mod"})
    }, {enabled:toggleDep});

    const addDependency=(e:React.FormEvent<HTMLFormElement>)=>{
        e.preventDefault();
        if(allModsQuery.data && addon.data){
            let newAddon = Object.assign({},addon.data);
            let masterList = allModsQuery.data.filter(({name})=>name !== newAddon.name);
            let newMod = masterList[e.currentTarget['newDep'].selectedIndex-1];
            if(!newAddon.dependencies.find(name =>  name === newMod.name) &&!(newAddon.name === newMod.name)){
                newAddon.dependencies.push(newMod.name);
                mutateAddon(newAddon)
            }
        }else{
            console.log(addon.data)
            console.log("else")
        }
    }
    const updateDependencies=(list:string[])=>{
        let newAddon = Object.assign({},addon.data);
        newAddon.dependencies = list
        mutateAddon(newAddon)
    }

    return(
        <div className={'w-2/3 mx-auto '}>
            {addon.data?.dependencies&&
                <DependencyList dependencies={addon.data?.dependencies} updateDependencies={updateDependencies}/>
            }
            <form onSubmit={addDependency} className={''}>
                <span className={'grid'}>
                    <button type={'button'} onClick={()=>{setToggleDep(!toggleDep)}}>{toggleDep?"Close Dependencies":"Add Dependency"}</button>
                    {
                        toggleDep&&
                        <span className={'flex'}>
                            <select name={"newDep"}>
                                <option>
                                </option>
                                {allModsQuery.data&&
                                    allModsQuery.data.map(mod=>{
                                        return(
                                            <option value={mod.name} key={mod.fileName}>
                                                {mod.name}
                                            </option>
                                        )
                                    })
                                }
                            </select>
                            <button className={'w-fit'} > Add to</button>
                        </span>
                    }
                </span>
            </form>
        </div>
    )
}