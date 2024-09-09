import {UseQueryResult} from "react-query";
import LoadingSpinner from "../LoadingSpinner.tsx";
import React from "react";

export default function ProfileList({title,profile_query,selectProfile,source}:Readonly<{title:string,profile_query:UseQueryResult<[string]>,selectProfile:React.MouseEventHandler,source:string}>){
    return(
        <div className={'bg-gray-700'}>
            <h3 className={'text-2xl text-center font-bold p-2'}>{title}</h3>
            {
                profile_query.isLoading?
                        <LoadingSpinner/>
                    :
                    profile_query.data?
                        profile_query.data.map(name=>{
                                return(<button type={'button'} onClick={selectProfile} key={"name"+name} name={`${source}:${name}`} id={name}>{name}</button>)
                            })
                        :
                        <>Something went wrong</>
            }
        </div>
    )
}