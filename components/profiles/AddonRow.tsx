import {ProfileAddon} from "../../lib/types";
import {useQuery} from "react-query";
import {invoke} from "@tauri-apps/api";
import {useState} from "react";

export default function AddonRow({addon,updateAddon}:Readonly<{ addon:ProfileAddon,updateAddon:(addon:ProfileAddon)=>void }>){
    const [toggleDep,setToggleDep] = useState(false)
    const allModsQuery=useQuery(["allMods"],async () => {
        return await invoke<ProfileAddon[]>("read_remote_mods")
    }, {enabled:toggleDep});
    const addDependency=(e:React.FormEvent<HTMLFormElement>)=>{
        e.preventDefault();
        if(allModsQuery.data){
            let masterList = allModsQuery.data.filter(({name})=>name !== addon.name);

            let newMod = masterList[e.currentTarget['newDep'].selectedIndex-1];
            if(!addon.dependencies.find(name =>  name === newMod.name) &&!(addon.name === newMod.name)){
                addon.dependencies.push(newMod.name);
                updateAddon(addon);
            }
        }
    }
    return(
        <form onSubmit={addDependency} className={'flex h-[3lh] w-full'}>
            <p>{addon.name}</p>
            <span className={'grid overflow-y-auto'}>
                {addon.dependencies.map(dep=>{
                    return (<span>{dep}</span>)
                })}
            </span>
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
                        <button className={'w-fit'} > Add</button>
                    </span>
                }
            </span>
        </form>
    )
}