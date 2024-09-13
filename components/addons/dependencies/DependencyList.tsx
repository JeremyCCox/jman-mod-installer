import React from "react";

export default function DependencyList({dependencies,updateDependencies}:Readonly<{dependencies:string[],updateDependencies:(newDependencies:string[])=>void}>){

    const removeDep=(e:React.MouseEvent<HTMLButtonElement>)=>{
        dependencies.splice(parseInt(e.currentTarget.value),1)
        updateDependencies(dependencies)
    }

    return(
        <span className={'grid overflow-y-auto min-h-8lh'}>
                {dependencies?.map((dep,index)=>{
                    return (
                        <div className={'w-full flex justify-between'} key={dep}>
                            <span>{dep}</span>
                            <button className={'w-fit px-8'} type={"button"} onClick={removeDep} value={index}> - </button>
                        </div>
                    )
                })}
            </span>
    )
}