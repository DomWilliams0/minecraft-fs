package ms.domwillia.mcfs.ipc

import com.google.gson.JsonElement
import com.google.gson.JsonPrimitive
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity

class NoGameException : Exception()
class UnknownCommandException : Exception()

object CommandExecutor {
    @Throws(NoGameException::class, UnknownCommandException::class)
    fun execute(command: String): JsonElement {
        when (command) {
            "PlayerHealth" -> return JsonPrimitive(thePlayer.health)
        }
        MinecraftFsMod.LOGGER.warn("Ignoring unknown command '$command'")
        throw UnknownCommandException()
    }

    private val thePlayer: ClientPlayerEntity
        get() = MinecraftClient.getInstance().player ?: throw NoGameException()

}