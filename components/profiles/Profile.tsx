import React from "react";

export default function Profile(){
    const copyProfile=async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>)=>{
        setLoading(true)
        e.preventDefault()
        setTimeout(()=>{
            setLoading(false)
        },7000)
        // console.log(await copyFile(`${path}\\profiles\\${e.currentTarget.name}`,`${path}\\profiles\\${e.currentTarget.name}-copy`,{recursive:true}));
    }
    return
}