import ProfileList from "./ProfileList.tsx";
import React, {useState} from "react";
import {useSearchParams} from "react-router-dom";
import {useQuery, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";

export default function ProfileLists(){
    const [toggle, setToggle]=useState(true)
    // @ts-ignore
    const [searchParams,setSearchParams] = useSearchParams();
    let local_profiles:UseQueryResult<[string]> = useQuery(["list_local_profiles"],async () => {
        return await invoke<[string]>("list_local_profiles").then(res=>{
            return res;
        }).catch(err=>{
            console.error(err)
            return([])
        })
    })
    let remote_profiles:UseQueryResult<[string]> = useQuery(["list_remote_profiles"],async () => {
        return await invoke<[string]>("list_remote_profiles").then(res=>{
            return res;
        }).catch(err=>{
            console.error(err)
            return([])
        })
    })
    const selectProfile=(e:React.MouseEvent<HTMLButtonElement>)=>{
        setSearchParams({ "source":e.currentTarget.name.split(":")[0],"profile": e.currentTarget.name.split(":")[1]});
    }
    return(
        <div >
            <button className={'w-20 px-0 mx-0 float-right'} onClick={()=>{setToggle(!toggle)}}>{toggle?"Hide":"Expand"}</button>
            <div className={'grid overflow-y-auto max-h-[60vh] transition-all duration-700'} style={{width:toggle?200:0}}>
                <ProfileList title={"Local Profiles"} profile_query={local_profiles} selectProfile={selectProfile} source={"local"}/>
                <ProfileList title={"Remote Profiles"} profile_query={remote_profiles} selectProfile={selectProfile} source={"remote"}/>
            </div>
        </div>
        )
}